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
| node | `v26.8.2` | `counts.json` |
| acorn | `8.18.0` | `counts.json` |
| kali binary | `kali 0.1.0` (`/workspace/.cache/cargo-target/debug/kali`) | `accepts.json` |
| §0.2's verdicts, measured at | `62b11a78c3` | `kali-silent-miscompile-register.md` §0.2's own sentence |
| this document generated at | `b7bfdfa178` | `git rev-parse HEAD`, recorded by the generator |
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
| R-57 (unclustered) | singleton | §2: 'unclustered, and deliberately not added to any of §3's eight ... not G1 ... not G8 ... and not N1, despite the name'. The N1 exclusion is explicit because of a word collision: N1 here is escape/provenance loss in the ARENA-ESCAPE sense (R-14/R-48), not the string-escape sense. |
| R-58 (unclustered) | singleton | §2: 'unclustered ... It has G3's shape exactly ... It is nonetheless not added to G3's member list, on G3's own criterion'. Declined because G3's stated remedy -- an allowlist at the choke point -- would refuse a program node runs. |
| R-64 (unclustered) | singleton | §2: 'unclustered -- traced to one specific codegen floor, not a shared cluster ... Not G6 ... Not G4 ... Not G2'. Declined G6 because the allocation is not unimplemented -- it materializes correctly in three named lanes and the callee side is proven innocent by a bound-argument control; declined G4 because the allocation produces a real, distinct handle, not a value collapsing into 0's own representation; declined G2 because no callee is unresolved. |
| R-65 (unclustered) | singleton | §2: 'unclustered ... this has G3's shape exactly ... It is nonetheless not added to G3's member list, on the criterion clusters.json itself states and that R-47 and R-58 already decided the same way ... Not G4 ... Not G6 ... Not G2'. Declined G3 on the fix-unit rule, the same criterion R-47 and R-58 already applied to their own G3-shaped guards; declined G4 because the array or constructed value produces a real, distinct handle wherever a materializing lane reaches it; declined G6 because an array literal and a user class constructor are fully implemented language features, not unresolved builtins; declined G2 because the callee is resolved and compiled in every repro. |
| R-67 (unclustered) | singleton | §2: 'unclustered -- traced to one specific codegen emission order ... Not G7 ... Not G4 ... Not G2'. Declined G7 because the shared word is 're-emit', but the site and trigger differ -- G7 re-emits a binding's own initializer at each read of that binding, while this entry re-emits the fill call's own argument expression once per loop iteration, independent of how or whether the result is ever bound; declined G4 because every call returns a real, distinct value and the defect is call count, not value representation; declined G2 because the callee is resolved and runs to completion every time. |

`aggregate` sums a cluster over its members, so an entry in two clusters would be counted twice; the assignment below is a partition. Where the register names two groups it names them in order, and the first is taken. Counts are per **entry**, not per lane, so a cluster sum carries an entry's whole frequency even where the register splits that entry across two clusters by lane.

| entry | tier | cluster | the register's own §2 line | the second reading, and why it was not taken |
|---|---|---|---|---|
| R-06 | 2 | G7 — binding storage: `const` has no cell, non-`const` composite initializers are lost | G7 (binding storage: `const` inlined, non-`const` composite initializers lost). | — |
| R-08 | 2 | G4 — there is no value distinct from the scalar `0` | G4 (no value distinct from scalar `0`). | §3's G3 member list also names R-08's `??` half. G3 asserts no shared code path, so the mechanism cluster is taken. |
| R-09 | 2 | R-09 (unclustered) | unclustered (isolated lowering bug). | — |
| R-10 | 2 | G7 — binding storage: `const` has no cell, non-`const` composite initializers are lost | G7. | — |
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
| R-34 | 4 | R-34 (unclustered) | not G8 (see below) -- currently unclustered. | §3's G8 member list still names R-34; §2's own entry refuses it. |
| R-47 | 2 | R-47 (unclustered) | unclustered. It has **G3**'s shape ... with a **G7** flavour ... deliberately *not* added to G3's member list. | G3 by shape, G7 by discriminator -- both named and both declined by the register. |
| R-48 | 2 | N1 — escape/provenance loss | unclustered (escape/provenance-loss family, with R-14). | — |
| R-51 | 1 | G2 — call lowering: unresolvable callee folds to constant `0` | G2 (call lowering: unresolvable callee folds to constant `0`) -- ... Recorded as G2 by symptom; the mechanism is named below. | By symptom only: the route is the optional-chain lowering, not an unresolvable callee. |
| R-52 | 1 | R-52 (unclustered) | unclustered (an isolated lowering/emit contract mismatch), but it is a textbook instance of the pattern §3's G-clusters keep circling. | — |
| R-53 | 2 | G4 — there is no value distinct from the scalar `0` | G4 (there is no value distinct from the scalar `0`) by symptom; plausibly G7 (binding storage) by mechanism, which is not traced. Recorded as G4. | G7, named and declined by the register. |
| R-57 | 2 | R-57 (unclustered) | unclustered, and deliberately not added to any of §3's eight ... not G1 ... not G8 ... not N1, despite the name ... It shares a shape with R-58 ... but they are not one cluster, by clusters.json's own definition that a cluster is the unit a fix actually ships in. | A shared cluster with R-58 (`kali_parser` converting a literal's source text with Rust's grammar) is named and declined by the register itself, on the fix-unit rule this file states. |
| R-58 | 2 | R-58 (unclustered) | unclustered, and the interesting part is which group it declines. It has G3's shape exactly ... It is nonetheless not added to G3's member list, on G3's own criterion ... A cluster is the unit a fix ships in; this fix does not ship with G3's. | G3 by shape, named and declined by the register because G3's remedy would refuse valid JavaScript; a shared cluster with R-57 is also named and declined, on the fix-unit rule. |
| R-60 | 2 | G6 — unresolved or unimplemented builtins fold to a default instead of failing closed | G6 -- unresolved or unimplemented builtins fold to a default instead of failing closed ... it joins on measurement rather than resemblance ... G6's raising-confidence experiment is what filing this entry ran ... G6's fix unit reaches it ... It is not G2 ... It is not G4. | G2 by the shared zero-placeholder fallback, named and declined by the register because G2's members are all user function values and a missing builtin is what G6 holds; G4 is also named and declined, because the property is PRESENT and there is a right answer for the read to have returned. |
| R-64 | 2 | R-64 (unclustered) | unclustered -- traced to one specific codegen floor, not a shared cluster; §3's existing clusters are declined by name rather than assumed. Not G6 ... Not G4 ... Not G2 ... The value is dropped and a type-plausible 0 is pushed in its place, with no diagnostic. | G6 by surface resemblance (a type-plausible zero instead of a diagnostic), named and declined by the register because the allocation is not an unimplemented builtin -- it materializes correctly through three named lanes and a bound-argument control proves the callee side innocent. |
| R-65 | 2 | R-65 (unclustered) | unclustered -- this has G3's shape exactly ... It is nonetheless not added to G3's member list, on the criterion clusters.json itself states and that R-47 and R-58 already decided the same way for their own G3-shaped guards: a cluster there is the unit a fix actually ships in, and this entry's fix ships in no code any G3 member's fix touches. Not G4 ... Not G6 ... Not G2. | G3 by shape (a guard keyed on one syntactic form, with a sibling form slipping past into the miscompile the guard's own diagnostic describes), named and declined by the register on the fix-unit rule R-47 and R-58 already established for their own G3-shaped guards. |
| R-67 | 2 | R-67 (unclustered) | unclustered -- traced to one specific codegen emission order, declined by name against §3's existing clusters rather than assumed. Not G7 (binding storage) ... the shared word is "re-emit", but the site and trigger are both different -- G7 re-emits a binding's own initializer at each read of that binding, keyed on declarator kind; this entry re-emits the fill call's own argument expression once per loop iteration, independent of how or whether the result is ever bound. Not G4 ... every call returns a real, distinct value; the defect is call count, not value representation. Not G2 ... the callee is resolved and runs to completion every time, not folding to a placeholder. | G7 by the shared word "re-emit", named and declined by the register because G7's mechanism is binding-storage re-emission on read, a different site and trigger from this entry's fill-loop emission order. |

### 2.2 The reachable axis — the ranking's own definition

Frequency is the count over the 126 corpus programs kali accepts, of which 126 are anchor micro-snippets. This is the axis the design spec §3 defines the ranking on, and in substance it is a ranking over test snippets: 0 of the 40 programs written to do a job rather than to probe the compiler is reachable.

**Band 1**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| G2 — call lowering: unresolvable callee folds to constant `0` | 1 | 0 | R-51 |
| R-52 (unclustered) | 1 | 0 | R-52 |
| G4 — there is no value distinct from the scalar `0` | 2 | n/a — uncountable member | R-08, R-21, R-53 |
| G5 — a string handle reaches a consumer that never proved it was a string | 2 | n/a — uncountable member | R-16, R-17, R-18 |
| R-22 (unclustered) | 2 | n/a — uncountable member | R-22 |
| G8 — per-sink rendering divergence: direct-log and concat are separate formatters | 2 | 59 | R-23, R-30, R-31 |

*Band 1 is contingent on the cluster assignment. §2.4 re-runs every contested assignment and finds two that move a band 1: R-21 (both axes) and R-23 (the reachable axis, by changing G8's worst tier). Quote this table with §2.4, not on its own.*

**Band 2**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| N1 — escape/provenance loss | 2 | 7 | R-14, R-48 |
| R-67 (unclustered) | 2 | 7 | R-67 |

**Band 3**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| R-65 (unclustered) | 2 | 5 | R-65 |

**Band 4**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| G7 — binding storage: `const` has no cell, non-`const` composite initializers are lost | 2 | 2 | R-06, R-10 |

**Band 5**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| G6 — unresolved or unimplemented builtins fold to a default instead of failing closed | 2 | 0 | R-15, R-24, R-25, R-60 |
| R-09 (unclustered) | 2 | 0 | R-09 |
| R-26 (unclustered) | 2 | 0 | R-26 |
| R-27 (unclustered) | 2 | 0 | R-27 |
| R-28 (unclustered) | 2 | 0 | R-28 |
| R-47 (unclustered) | 2 | 0 | R-47 |
| R-57 (unclustered) | 2 | 0 | R-57 |
| R-58 (unclustered) | 2 | 0 | R-58 |
| R-64 (unclustered) | 2 | 0 | R-64 |

**Band 6**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| R-34 (unclustered) | 4 | 0 | R-34 |

**Countable-only band 1** (the same computation with every uncountable cluster dropped rather than carried, so a reader can see the measured frontier on its own): G2 — call lowering: unresolvable callee folds to constant `0` (tier 1, 0); R-52 (unclustered) (tier 1, 0); G8 — per-sink rendering divergence: direct-log and concat are separate formatters (tier 2, 59).

### 2.3 The raw axis — published beside it, never instead of it

The same clusters banded on the count over all 177 corpus programs, accepted or not. This is the axis that carries what the extension stratum says, because 40 of its 40 programs are unreachable. It is published so a reader can see how far the reachability gate moved each cluster; it is NOT a substitute for the reachable axis, and the corpus is not widened to make the two agree (spec §4.3 forbids adjusting the corpus once scores are visible).

**Band 1**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| R-52 (unclustered) | 1 | 5 | R-52 |
| G4 — there is no value distinct from the scalar `0` | 2 | n/a — uncountable member | R-08, R-21, R-53 |
| G5 — a string handle reaches a consumer that never proved it was a string | 2 | n/a — uncountable member | R-16, R-17, R-18 |
| R-22 (unclustered) | 2 | n/a — uncountable member | R-22 |
| N1 — escape/provenance loss | 2 | 99 | R-14, R-48 |

*Band 1 is contingent on the cluster assignment. §2.4 re-runs every contested assignment and finds two that move a band 1: R-21 (both axes) and R-23 (the reachable axis, by changing G8's worst tier). Quote this table with §2.4, not on its own.*

**Band 2**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| G2 — call lowering: unresolvable callee folds to constant `0` | 1 | 3 | R-51 |
| G8 — per-sink rendering divergence: direct-log and concat are separate formatters | 2 | 79 | R-23, R-30, R-31 |

**Band 3**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| R-65 (unclustered) | 2 | 58 | R-65 |

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
| R-67 (unclustered) | 2 | 12 | R-67 |

**Band 7**

| cluster | worst tier | frequency | members |
|---|---|---|---|
| G6 — unresolved or unimplemented builtins fold to a default instead of failing closed | 2 | 3 | R-15, R-24, R-25, R-60 |
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
| R-57 (unclustered) | 2 | 0 | R-57 |
| R-58 (unclustered) | 2 | 0 | R-58 |
| R-64 (unclustered) | 2 | 0 | R-64 |

**Countable-only band 1** (the same computation with every uncountable cluster dropped rather than carried, so a reader can see the measured frontier on its own): R-52 (unclustered) (tier 1, 5); N1 — escape/provenance loss (tier 2, 99).

### 2.4 How much the contested assignments matter

13 of the 31 ranked entries have a second cluster the register names with a concrete destination. Each is moved to it, alone, and both band 1s are recomputed. A clustering that cannot be argued with is not a measurement, so the argument is priced here rather than asserted away.

| entry | assigned | moved to | reachable band 1 | raw band 1 |
|---|---|---|---|---|
| R-08 | G4 — there is no value distinct from the scalar `0` | G3 — guards whose own diagnostic text names the unsoundness that leaks past them | unchanged | unchanged |
| R-15 | G6 — unresolved or unimplemented builtins fold to a default instead of failing closed | G5 — a string handle reaches a consumer that never proved it was a string | unchanged | unchanged |
| R-18 | G5 — a string handle reaches a consumer that never proved it was a string | G3 — guards whose own diagnostic text names the unsoundness that leaks past them | unchanged | unchanged |
| R-21 | G4 — there is no value distinct from the scalar `0` | G8 — per-sink rendering divergence: direct-log and concat are separate formatters | unchanged | gains **G8 — per-sink rendering divergence: direct-log and concat are separate formatters**; loses **G4 — there is no value distinct from the scalar `0`** |
| R-23 | G8 — per-sink rendering divergence: direct-log and concat are separate formatters | G4 — there is no value distinct from the scalar `0` | gains **N1 — escape/provenance loss**, **R-67 (unclustered)** | unchanged |
| R-28 | R-28 (unclustered) | G8 — per-sink rendering divergence: direct-log and concat are separate formatters | unchanged | unchanged |
| R-34 | R-34 (unclustered) | G8 — per-sink rendering divergence: direct-log and concat are separate formatters | unchanged | unchanged |
| R-47 | R-47 (unclustered) | G3 — guards whose own diagnostic text names the unsoundness that leaks past them | unchanged | unchanged |
| R-53 | G4 — there is no value distinct from the scalar `0` | G7 — binding storage: `const` has no cell, non-`const` composite initializers are lost | unchanged | unchanged |
| R-58 | R-58 (unclustered) | G3 — guards whose own diagnostic text names the unsoundness that leaks past them | unchanged | unchanged |
| R-60 | G6 — unresolved or unimplemented builtins fold to a default instead of failing closed | G2 — call lowering: unresolvable callee folds to constant `0` | unchanged | unchanged |
| R-64 | R-64 (unclustered) | G6 — unresolved or unimplemented builtins fold to a default instead of failing closed | unchanged | unchanged |
| R-67 | R-67 (unclustered) | G7 — binding storage: `const` has no cell, non-`const` composite initializers are lost | unchanged | unchanged |

## 3. The per-entry table

Every input to §2, so a reader who disagrees with the clustering can re-band from here. `raw` counts all 177 programs; `reachable` counts only the 126 kali accepts. `zero` names WHICH KIND of zero a zero is — the three are not the same claim and must never be pooled (`counts.json` `zeroKinds`).

| entry | tier | raw | reachable | anchor raw/reach | extension raw/reach | §0.2 lanes | zero kind | upper bound | cluster |
|---|---|---|---|---|---|---|---|---|---|
| R-06 | 2 | 6 | 1 | 4 / 1 | 2 / 0 | FIXED / SILENT / SILENT | — | — | G7 — binding storage: `const` has no cell, non-`const` composite initializers are lost |
| R-08 | 2 | 95 | 14 | 14 / 14 | 81 / 0 | FAIL_CLOSED / SILENT | — | yes (disclosed in record) | G4 — there is no value distinct from the scalar `0` |
| R-09 | 2 | 16 | 0 | 0 / 0 | 16 / 0 | SILENT / FL_INTERNAL | present-but-unreachable | — | R-09 (unclustered) |
| R-10 | 2 | 11 | 1 | 1 / 1 | 10 / 0 | SILENT | — | — | G7 — binding storage: `const` has no cell, non-`const` composite initializers are lost |
| R-14 | 2 | 99 | 7 | 8 / 7 | 91 / 0 | SILENT | — | yes (**not** disclosed in record) | N1 — escape/provenance loss |
| R-15 | 2 | 0 | 0 | 0 / 0 | 0 / 0 | FAIL_CLOSED / SILENT | unsampled | — | G6 — unresolved or unimplemented builtins fold to a default instead of failing closed |
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
| R-34 | 4 | 4 | 0 | 0 / 0 | 4 / 0 | SILENT | present-but-unreachable | — | R-34 (unclustered) |
| R-47 | 2 | 1 | 0 | 0 / 0 | 1 / 0 | SILENT / FAIL_CLOSED / FIXED | present-but-unreachable | — | R-47 (unclustered) |
| R-48 | 2 | 0 | 0 | 0 / 0 | 0 / 0 | SILENT | unsampled | — | N1 — escape/provenance loss |
| R-51 | 1 | 3 | 0 | 0 / 0 | 3 / 0 | SILENT | present-but-unreachable | — | G2 — call lowering: unresolvable callee folds to constant `0` |
| R-52 | 1 | 5 | 0 | 0 / 0 | 5 / 0 | SILENT / FL_INTERNAL | present-but-unreachable | — | R-52 (unclustered) |
| R-53 | 2 | 1 | 0 | 0 / 0 | 1 / 0 | SILENT / FIXED | present-but-unreachable | — | G4 — there is no value distinct from the scalar `0` |
| R-57 | 2 | 0 | 0 | 0 / 0 | 0 / 0 | SILENT | unsampled | yes (disclosed in record) | R-57 (unclustered) |
| R-58 | 2 | 0 | 0 | 0 / 0 | 0 / 0 | SILENT | unsampled | yes (disclosed in record) | R-58 (unclustered) |
| R-60 | 2 | 0 | 0 | 0 / 0 | 0 / 0 | SILENT | unsampled | yes (**not** disclosed in record) | G6 — unresolved or unimplemented builtins fold to a default instead of failing closed |
| R-64 | 2 | 0 | 0 | 0 / 0 | 0 / 0 | SILENT | unsampled | yes (disclosed in record) | R-64 (unclustered) |
| R-65 | 2 | 58 | 5 | 5 / 5 | 53 / 0 | SILENT | — | yes (disclosed in record) | R-65 (unclustered) |
| R-67 | 2 | 12 | 7 | 7 / 7 | 5 / 0 | SILENT | — | yes (disclosed in record) | R-67 (unclustered) |

### 3.1 What the SILENT filter removed, and what it cost the ranking

Spec §8.1 removes these 20 entries for **two different reasons**, and collapsing them would misdescribe 2 of them:

- **Not damage** — `FIXED`, `FAIL_CLOSED`, `BOTH_REJECT`. kali either agrees with node or refuses honestly. 18 entries leave this way: R-01, R-02, R-03, R-04, R-05, R-07, R-11, R-12, R-13, R-19, R-20, R-32, R-33, R-49, R-56, R-59, R-63, R-66.
- **Outside this ranking's question** — `ACCEPTS_INVALID`, `FL_INTERNAL`, `TIMEOUT`, `NONDETERMINISTIC`. §8.1 *reports* these in the regenerated table and keeps them out of the ranking, whose question is *what silent defect should be fixed next*. 2 entries leave this way: R-29, R-54. The distinction is not pedantic: R-29's §0.2 row records kali printing `r=1` at exit 0 with no diagnostic, which is silent by any plain reading. It is out because accepting a program node rejects is a different defect class from giving a wrong answer to a valid one — not because nothing bad happens.

Their counts are printed because the removal is not cosmetic: it takes the largest reachable count in the whole measurement out of the ranking.

| entry | tier | raw | reachable | §0.2 lanes |
|---|---|---|---|---|
| R-07 | 2 | 449 | 78 | FIXED |
| R-13 | 2 | 302 | 43 | FIXED |
| R-59 | 2 | 235 | 25 | FAIL_CLOSED |
| R-01 | 1 | 18 | 0 | FAIL_CLOSED |
| R-02 | 1 | 2 | 0 | FAIL_CLOSED |
| R-03 | 1 | 15 | 0 | FAIL_CLOSED |
| R-04 | 1 | 112 | 0 | FIXED |
| R-05 | 1 | 6 | 0 | FAIL_CLOSED |
| R-11 | 2 | 10 | 0 | FIXED |
| R-12 | 2 | 3 | 0 | FAIL_CLOSED |
| R-19 | 2 | 27 | 0 | FIXED / FAIL_CLOSED |
| R-20 | 2 | 4 | 0 | FAIL_CLOSED |
| R-29 | 3 | uncountable | uncountable | ACCEPTS_INVALID |
| R-32 | 4 | 6 | 0 | FIXED |
| R-33 | 4 | 17 | 0 | FIXED |
| R-49 | 1 | 2 | 0 | FAIL_CLOSED |
| R-54 | 3 | uncountable | uncountable | ACCEPTS_INVALID |
| R-56 | 2 | 0 | 0 | FIXED |
| R-63 | 2 | 35 | 0 | FAIL_CLOSED |
| R-66 | 2 | 0 | 0 | FAIL_CLOSED |

Only 3 of the 20 removed entries have a nonzero reachable count at all: R-07 (78) and R-13 (43) and R-59 (25). The largest of them, R-07 at 78, is **the largest reachable count anywhere in `counts.json`** — larger than the largest that survives the filter (R-30 at 57). The ranking's numeric input is much thinner than the raw measurement looks.

And of the 31 entries that do enter, **8 have a reachable count above zero** (R-06 = 1, R-08 = 14, R-10 = 1, R-14 = 7, R-30 = 57, R-31 = 2, R-65 = 5, R-67 = 7); 20 measure zero and 3 have no count at all. The bands below separate 20 clusters on the evidence of 8 nonzero entries.

### 3.2 R-13's number is not R-13's shape

The register's R-13 repro is an **object read with a variable key**. `computedMemberNonLiteralKey` counts every computed member access with a non-literal key, which includes ordinary array indexing `a[i]` — and array indexing demonstrably works. The committed breakdown in `counts.json` splits the same sites by receiver and by position:

| axis | total | object-literal receiver | array-like receiver | store target |
|---|---|---|---|---|
| raw (all programs) | 302 | 56 | 45 | 67 |
| reachable (pooled) | 43 | 0 | 26 | 18 |
| reachable — anchor | 43 | 0 | 26 | 18 |
| reachable — extension | 0 | 0 | 0 | 0 |

Read down the reachable rows: of R-13's 43 reachable sites, **0 have the object-literal receiver the register's repro describes**, and the anchor's share of those is **0** — so **all 43 reachable anchor sites have none**. Both register-shaped sites are in the extension stratum, whose reachable population is the 0 program kali accepts. A further **18 of the 43 are store targets**, not reads: the register treats the write half as the worse one, but it is a different site class from the read its repro shows. R-13's 43 is an upper bound on a construct family, not a count of how often R-13's defect is triggered.

### 3.3 Which counts are upper bounds

A count is an upper bound when the predicate admits sites the defect does not reach — because the AST cannot see a runtime type, a representation, or a compiler-internal proof. 12 records disclose their own upper bound: R-08, R-16, R-26, R-30, R-56 (not in the ranking), R-57, R-58, R-59 (not in the ranking), R-63 (not in the ranking), R-64, R-65, R-67. 4 more are upper bounds their records do **not** disclose, found by this measurement: R-07 (not in the ranking), R-13 (not in the ranking), R-14, R-60. Every note is in `counts.json` under `upperBound`.

### 3.4 A lane result is not an entry result

10 of the 31 ranked entries measure something other than SILENT on at least one lane, and none of them is thereby retired: R-06 (FIXED / SILENT / SILENT); R-08 (FAIL_CLOSED / SILENT); R-09 (SILENT / FL_INTERNAL); R-15 (FAIL_CLOSED / SILENT); R-21 (FAIL_CLOSED / SILENT); R-25 (FAIL_CLOSED / SILENT); R-30 (SILENT / FIXED); R-47 (SILENT / FAIL_CLOSED / FIXED); R-52 (SILENT / FL_INTERNAL); R-53 (SILENT / FIXED). **This list can never hold a retired entry, by construction.** It is built from the entries the SILENT filter admits, and an entry whose every lane has moved has no SILENT lane left, so it is removed by that filter before this list is assembled and leaves the ranking altogether rather than appearing here as all-FIXED. §3.1 is where such an entry surfaces. R-33 left exactly that way on 2026-08-16 — its `console.warn` lane moved and its `console.error` control was already FIXED — and it is genuinely retired. R-32 left by the same door on the same day and is **not** retired: leaving is a statement about the dangerous class, not about being fixed, and §0.2's R-32 row records the `1e21` binding and concat behaviour that still holds it open with no live case pinning it. §0.2 records why in each remaining case — R-47's and R-53's FIXED lanes are the `const` controls those entries declare for themselves. R-30's six FIXED case-lanes are of two kinds, and the difference is worth keeping: four are controls the entry declares for itself (its `const`-scalar lane, its concat/template sinks, and the taint-reaching and proven-`String()` guards at the single-argument sink), while two — its `const`-object-field lane and the taint-reaching `String()`-result lane at the MULTI-argument sink — genuinely moved on 2026-08-16. So *declared control* is the accurate description of the first four and *`const` lane* is not, and neither kind retires the entry, because its plain `var`-binding lane is still SILENT. R-08's `===` half fails closed while its `??` half is **still SILENT**, unchanged by that move. R-49 — not in the ranking at all — fails closed by **R-35's** switch allowlist rather than by its own gate. An entry is retired when every lane moves, which is a claim no single lane can make.

## 4. The uncountable entries

No frequency exists for these, so they are banded on **tier alone** and are never merged into §2's numeric bands. An uncountable entry is not a rare one: it is one the counter cannot see at all, and publishing it as `0` would rank it below every entry the corpus happens to contain.

| entry | tier | in the ranking? | kind | why no count exists |
|---|---|---|---|---|
| R-17 | 2 | yes — SILENT | no syntactic predicate (representation- or runtime-typed) | a representation condition of the same G5 family as R-16 -- a string handle reaching a consumer that never proved it was a string; whether an array element or `Object.keys` result is a string is a repr fact, not a syntactic one |
| R-21 | 2 | yes — SILENT | no syntactic predicate (representation- or runtime-typed) | a representation condition -- there is no `undefined` distinct from scalar `0`, so it fires wherever an expression *evaluates* to absent or void (a missing field, an out-of-range read, a void return), which is a runtime-value fact rather than a construct in the source |
| R-22 | 2 | yes — SILENT | no syntactic predicate (representation- or runtime-typed) | a runtime-type condition -- the missing rung is number/string coercion, so it fires only when the two operands actually hold a number and a string at run time; same-type `==` comparisons are correct, and the operator alone does not identify the case |
| R-29 | 3 | no — removed by the SILENT filter | structurally uncountable | An assignment to a `const` binding is a TypeError at run time, so no program that runs clean under node can execute one; the construct and this corpus's runnability requirement are mutually exclusive (corpus/README.md). This zero is not a frequency and must never be ranked as one. |
| R-54 | 3 | no — removed by the SILENT filter | no syntactic predicate (representation- or runtime-typed) | only invalid JavaScript triggers it -- acorn, like node, rejects a second `default` clause as a SyntaxError, so the shape can never appear in a corpus file that parses |
| R-61 | 2 | no — removed by the SILENT filter | no syntactic predicate (representation- or runtime-typed) | a BUILD-TIER condition, not a syntactic one -- the triggering source (`new Array(n)`, optionally `.fill(v)`, later read by a literal index) is identical text whether it compiles correctly at --fast or wrongly at --release/--release-advanced, so no acorn-visible AST shape distinguishes a corpus file this matters for from one it does not; the corpus matchers run once over source text and have no concept of build tier |

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
| extension | 0 | 40 | 0.0% |

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

> 126 of 177 programs are reachable, and 126 of those 126 are ANCHOR programs -- a stratum that is 131 micro-snippets written to probe compiler behaviour plus 6 real CLBG programs. Every reachable ranking is therefore, in substance, a ranking over test snippets. Read the per-entry `strata` split before treating any reachable figure as a frequency in real code.

> 0/40 extension programs are accepted (0.0%). The extension is the stratum written to do jobs rather than to probe the compiler, so almost everything it measures about real programs lands in the RAW column only. Its accept rate is a finding in its own right, not a defect of the corpus: curation was independent of acceptance.

> The extension is written in the project's imperative-core dialect: no regex, no destructuring, no template literals, no `??`, no class/Map/Set/async. See corpus/README.md for which counts that biases and in which direction. A frequency here is a frequency in *programs of that dialect*, not in JavaScript generally.
<!-- GENERATED:END -->

## 6. Commentary — authored, not generated

**This section is written by hand.** Nothing in it is computed, and where it
argues it says so. Sections 2 to 5 are the measurement; this section is what one
reader thinks the measurement means, and a later reader is free to disagree with
it without disturbing a single number above.

**AMENDMENT 2026-08-16, at `62b11a78c3` — the first regeneration driven by a
change from outside the ranking project, and it moved §2.** The
console-render-unification project
(`docs/superpowers/specs/2026-08-15-console-render-unification-design.md`) moved
three §0.2 rows. **R-33 retired** — every lane of it moved, which is §3.4's rule.
**R-32's live lanes are all FIXED**, though its §0.2 row records that it is *not*
retired: behaviour its own title claims is still broken at `1e21` in the binding
and concat lanes, pinned by no live case. Both therefore lose their SILENT lane,
leave the SILENT filter, and are removed from `tools/blast-radius/clusters.json`,
where they were **G8** members. **R-30 did not retire and did not leave**: its
plain `var`-binding lane is still SILENT, so G8 keeps R-30's 57 — the largest
reachable count that survives the filter — and G8 stays in the ranking.

Every figure in the amendments below is **read out of the regenerated §2 and
§3**, not predicted; §6.6 item 4's instruction is *re-run, do not re-read*, and
this is the first time an outside change has exercised it. What moved, verbatim
from the regeneration:

- **G8's reachable frequency: ~~65~~ → 59**, members ~~R-23, R-30, R-31, R-32,
  R-33~~ → **R-23, R-30, R-31**. It keeps its place in reachable band 1 and still
  dominates G3 (tier 2, 45).
- **G8's raw frequency: ~~102~~ → 79, and it changed BAND**: raw band 2 is now
  **N1** (escape/provenance loss, 99) alone, and G8 drops to raw band 3. This is
  the largest structural movement in the regeneration and it is not visible from
  the reachable axis at all — an argument for §1.1's rule that the two axes are
  published side by side.
- **Ranked entries: ~~29~~ → 27.** Entries removed by the SILENT filter:
  ~~12~~ → 14, of which the *not damage* group goes ~~10~~ → 12 (R-32 and R-33
  join it). Entries entering with a nonzero reachable count: ~~9~~ → **7**, so
  §2's 16 clusters are now separated on the evidence of seven entries.
- **A contested assignment changed its verdict.** In §2.4, moving **R-08** from
  G4 to G3 was previously priced as `unchanged` on both axes. It now *gains G3
  and loses G8* on the reachable band 1. Nothing about R-08 moved; G8's frequency
  fell from 65 to 59 and R-08's alternative destination G3 sits at 45, so the
  dominance relation between them became sensitive to a swap it was not sensitive
  to before. **This is a real, and slightly alarming, demonstration of §6.3's
  point**: the frontier's shape is not robust, and fixing an unrelated tier-4
  entry was enough to make one more contested assignment load-bearing.

What did **not** happen, recorded because the project's own spec predicted it
would: G8 does **not** lose R-30's 57, because R-30 does not retire. Any reading
of this document that expected G8 to collapse should be discarded.

**AMENDMENT 2026-08-16, at `3a636f62fb` — a SECOND regeneration on the same day,
and the first one driven by a register entry that did not exist before.** The
console-render-unification project's final whole-branch review found a new silent
miscompile in the codegen predicate that project had just narrowed, and filed it
as **R-56** (§2, Tier 2 — a string key spelled `'"5"'` is indistinguishable from
the numeric key `5`, so `Object.hasOwn` denies a property the same program has
just read). A §2 entry with a SILENT §0.2 row is a ranked entry, so this document
regenerated with it. Read out of the regenerated §2 and §3, not predicted:

- **Ranked entries: ~~27~~ → 28**, and **§2's clusters ~~16~~ → 17** — R-56 takes
  a new singleton cluster `R-56 (unclustered)`. G3 is named as its shape and
  declined, for the reason §2's entry gives, so R-56 also appears in §2.4's
  contested-assignment table, where **both axes price the swap `unchanged`**.
- **No band moved, on either axis.** R-56 is **countable** and measures **0
  reachable / 0 raw** over the frozen corpus (zero-kind `unsampled`), so it lands
  in reachable band 5 and raw band 9 among the other tier-2 zeros, and dominates
  nothing. Band 1 is byte-identical to the block above on both axes. That is the
  whole of the structural movement, and it is worth contrasting with the first
  amendment's: a *fix* moved a band, and a *new defect* did not.
- **The instrument moved, which has not happened before.** R-56's predicate is
  new, so `predicates.json` gained a record and `matchers.mjs` gained the matcher
  it names; both frozen SHAs in
  `crates/kali_blast_radius/src/manifest_tests.rs` were re-pinned and
  `counts.json` was regenerated with them. The regeneration's diff is the
  evidence that nothing else moved: it added exactly one entry and left every
  other entry's every field byte-identical. Per **spec §4.3** the instrument
  change is **its own commit**, landed ahead of the finding that motivated it, so
  a reader bisecting a moved figure lands on a commit that changes nothing but
  the apparatus.
- **The `countable` word in that record is load-bearing, and nothing checks it.**
  `score::aggregate` makes a cluster uncountable if any member is, and an
  uncountable cluster is never dominated — so filing R-56 `uncountable` would
  have been easier (no matcher, one SHA instead of two) and would have placed a
  brand-new zero-frequency entry in **band 1** by construction, with no gate
  objecting. It is filed countable because its triggering shape is syntax, which
  is the only reason available. `manifest_tests.rs`'s frozen-SHA comment carries
  the same warning for the next author.
- **`upperBound` disclosures: ~~4~~ → 5.** R-56's record discloses its own: the
  exact condition is that the key's inner text lies in Rust's `Display for f64`
  image, which an acorn AST cannot reproduce, so the matcher tests a strictly
  wider "reads as a number" instead.

**What this amendment does NOT fix, and it is the more consequential of the two
notes on this page.** The register's §0.2 movement paragraph now discloses that
the SILENT set excludes a **measured silent lane** — `var x = 1e21;
console.log(x)` prints 22 digits at exit 0 and is tracked under **R-55** in §7
*"Fail-loudly-but-wrong defects (not silent)"*, which carries no §0.2 row and
therefore no oracle case. **This document's SILENT population is short by one
measurable lane, and no gate can notice**, because the gate compares §0.2 rows to
cases and that lane has neither. Splitting R-55 is filed as a follow-up in the
register; until it happens, ~~"28 ranked entries"~~ **"27 ranked entries"**
(re-counted 2026-08-16 at `12fd424897`; see the amendment below) is a count of
what is *rowed*, not of what is *silent*. **That disclosure is unchanged and
still live** — R-55 has not been split, so this document's SILENT population is
still short by one measurable lane whatever the ranked-entry count reads.

**AMENDMENT 2026-08-16, at `12fd424897` — a THIRD regeneration on the same day,
and the first driven by a register entry RETIRING.** The
hir-property-key-identity project
(`docs/superpowers/sdd/2026-08-16-hir-property-key-identity/`) closed **R-56**,
the entry the amendment immediately above had just added. `lower_property_name`
now stores `String(key)` for a numeric key instead of wrapping it in double
quotes, so `{5: 1}` and `{'"5"': 1}` no longer reach codegen as the same text;
both `r56a` cases were re-measured against `node v26.7.0` in both scopes, read
FIXED (kali prints `1`, `true`, `false` at exit 0 with empty stderr; node prints
the same three lines at exit 0), and were flipped `silent` → `fixed` only after
that reading. R-56 is **retired**, not merely departed — its single lane is its
whole title, its `hasOwn` 2x2 with both member-read controls now agrees with node
in all six cells, and the mechanism is gone rather than narrowed. So it loses its
SILENT lane, leaves the SILENT filter, and is removed from
`tools/blast-radius/clusters.json` — **both** its assignment and its
`R-56 (unclustered)` singleton cluster definition, which is the difference from
R-32's and R-33's removal above: those were **G8** members and G8 survives them,
while a singleton's cluster has no other member to keep it alive.

Every figure below is **read out of this regeneration's stdout**, spliced
verbatim into §2–§5 above; §6.6 item 4's instruction is *re-run, do not re-read*,
and the figures were taken from the generator's own output rather than from the
task's plan, which had predicted them.

- **Ranked entries: ~~28~~ → 27**, and **§2's clusters ~~17~~ → 16** (§3.1: "the
  bands below separate 16 clusters on the evidence of 7 nonzero entries"). Both
  moves are exactly the reverse of the amendment above, which is what a retiring
  entry with no co-members should do.
- **No band moved, on either axis, and both band 1s are byte-identical.** R-56's
  singleton cluster left reachable band 5 and raw band 9 — the tier-2 zeros — and
  every other cluster's band membership is unchanged line for line. Checked by
  differencing the band blocks of the pre- and post-splice documents, not by
  eye: reachable band 5 goes 7 cluster rows → 6 and raw band 9 goes 3 → 2, both
  losing only the `R-56 (unclustered)` row, and no other band's membership
  changes by a line. That R-56 measured **0 reachable / 0
  raw** is why: a cluster that dominates nothing takes nothing with it when it
  leaves.
- **Entries removed by the SILENT filter: ~~14~~ → 15**, of which the *not
  damage* group goes ~~12~~ → 13 (R-56 joins it, at **FIXED**, with reachable 0
  and raw 0). §3.1's "Only 4 of the 15 removed entries have a nonzero reachable
  count at all" is unchanged in its list — R-04, R-07, R-32, R-33 — because R-56
  contributes a zero to the denominator only.
- **Entries entering with a nonzero reachable count: 7, UNCHANGED**, and the same
  seven (R-06, R-08, R-10, R-13, R-14, R-30, R-31). The zeros go ~~18~~ → 17 and
  the no-count entries stay at 3. Nothing about the ranking's numeric evidence
  moved; only its denominator did.
- **Contested assignments: ~~10~~ → 9** in §2.4. R-56's own contested row (its
  declined **G3** alternative, priced `unchanged` on both axes) left with it. **No
  surviving contested assignment changed its verdict** — unlike the first
  amendment above, where G8's fall from 65 to 59 made moving R-08 load-bearing.
  That is the expected result here, since no cluster's frequency moved at all.
- **`upperBound` disclosures: still 5**, but R-56's is now annotated
  `(not in the ranking)` by the generator, joining R-07 — the count is derived
  from `counts.json` rather than from the SILENT filter, so a retiring entry's
  disclosure survives its departure. `predicates.json`, `matchers.mjs` and
  `counts.json` were **not** touched by this regeneration: the instrument is
  unchanged, so unlike the amendment above there is no §4.3 apparatus commit and
  no frozen SHA moved. The matcher for a retired entry's shape is deliberately
  kept, so that a regression re-lights the same number.

**Whether the figures matched what was predicted, stated plainly because the
whole point of this section is that it must not be assumed.** The task's spec
predicted **28 → 27 ranked entries, 17 → 16 clusters, and no band movement**, on
the reasoning that R-56 measures 0 reachable / 0 raw. **All three predictions were
correct**, and each was nonetheless re-read out of the generator's stdout rather
than copied from the spec. Two figures the spec did not predict also moved and are
recorded above because a regeneration's whole diff is the finding, not the part
that was anticipated: the SILENT filter's removal count (14 → 15, *not damage*
12 → 13) and §2.4's contested-assignment count (10 → 9).

**One thing was deliberately NOT changed, and a reader should know why.** §3.4's
generated prose names R-32 and R-33 as the two entries that left through the
SILENT filter on 2026-08-16 and does not name R-56, which left through the same
door on the same day. That sentence is illustrative rather than exhaustive and is
still true as written, and §3.4 already points a reader at §3.1 as the place such
an entry surfaces — where R-56 now appears, at FIXED. Editing it would mean
changing `crates/kali_blast_radius/src/ranking.rs`, which is the generator, and
this task changed no apparatus; the R-33 retirement edited that prose only
because it had gone false. Recorded here so the omission reads as a decision
rather than an oversight.

**Two live divergences were found while measuring this retirement and are NOT
filed.** Neither is R-56.

- **`.length` on a string read out of an ARRAY ELEMENT diverges.**
  `["abc"][0].length` prints `2` where node prints `3` — a program with **no
  `Object.keys` and no object key in it at all**, so this is not a property-key
  phenomenon. `Object.keys(o)[0].length` is one spelling of it; the *iteration*
  lane `for (const k of Object.keys(o)) k.length` is **correct** and matches
  node. kali prints `2` for every string tried (`["a"]`, `["ab"]`, `["abc"]`,
  `["abcdefghij"]`), so the value does not track the string. **This is very
  likely the family of R-17** (**G5** — *"String handles escape as raw integers
  from the plain-array and `Object.keys` lanes"*), the same lane at a different
  consumer (`.length` rather than concat); R-17's `Object.keys` repro records
  that `k.length` is correct, but that is the **array's** length. Whether it
  belongs under R-17, under R-16, or in a new entry is a human's decision and is
  not made here.
- **`Object.keys({"\"5\"": 1})[0]`** prints `\"5\"` where node prints `"5"` — the
  escape-sequence exception already recorded at `de87e48e8e` and in
  `docs/superpowers/followups/property-key-trim-site-classification.md` §6, whose
  cause is the parser not decoding escapes.

Both are noted in the register's §0.2 movement bullet and in §2's R-56 entry.
**They are not in this ranking's figures**, because an unfiled defect has no §0.2
row, no predicate record and no count — which is the same structural blind spot
the R-55 note above describes, met a second time.

**AMENDMENT 2026-09-08, at ~~`dde0f083c0`~~ `5ca1588cd1` — a FOURTH regeneration,
and the first driven by a filing decision rather than by a code change.**
(Corrected 2026-09-08 in final review: `dde0f083c0` is this branch's BASE and
carries no regeneration. This regeneration's own provenance row records
`5ca1588cd1`, the apparatus commit the generator was run at, and it landed in
`b13c890330`.) The
register-property-key-followups branch filed **R-57** (§2, Tier 2 — a property
key spelled with an escape sequence is stored undecoded, so the object has no
property under the name JavaScript denotes) and **R-58** (§2, Tier 2 — a
legacy-octal numeric key is read as decimal, so `{042: 1}` is the property `42`
where JavaScript says `34`). Nothing in the compiler moved: both divergences were
live at the previous regeneration and are live now. What moved is that two of the
four divergences the amendment above records as "found ... and NOT filed" have
been filed, at the human's instruction, so they acquire a §0.2 row, a predicate
record and a count and enter this document for the first time. Read out of the
regenerated §2 and §3, not predicted:

- **Ranked entries: ~~27~~ → 29**, and **§2's clusters ~~16~~ → 18** — R-57 and
  R-58 each take a new singleton cluster, `R-57 (unclustered)` and
  `R-58 (unclustered)`. They are **deliberately not merged into one cluster**
  despite sharing a shape (both are `kali_parser` converting a literal's source
  text with Rust's grammar where JavaScript's is required); the reason is
  `clusters.json`'s own definition that a cluster is *the unit a fix ships in*,
  and these two fixes cannot land in one change. Merging them would have made
  this line read `→ 17` and would have been the tidier ranking; it is declined on
  the file's own rule and the decline is recorded in that file's `note`.
- **No band moved, on either axis.** Both entries are **countable** and both
  measure **0 reachable / 0 raw** over the frozen corpus (zero-kind `unsampled`,
  anchor 0/0 and extension 0/0), so both land in reachable band 5 and raw
  band 9 — the same two bands R-56 landed in — and dominate nothing. Band 1 is
  byte-identical to the block above on both axes. That is now twice that a *new
  defect* has moved no band while a *fix* moved one.
- **§3.1's arithmetic: the entries entering with a nonzero reachable count is
  still 7**, unchanged; what moved is the denominator and the zero column —
  "17 measure zero" becomes **19**, and "16 clusters ... on the evidence of 7
  nonzero entries" becomes **18 clusters on the evidence of 7**. The ranking's
  numeric input did not grow at all; only the number of things it has to separate
  did. That is the honest reading of adding two zero-frequency entries, and it is
  the second time this document has had to state it.
- **§2.4's contested assignments: ~~9 of 27~~ → 10 of 29.** R-58 enters that
  table because its entry names **G3** as its shape and declines it; the swap
  prices `unchanged` on both axes. R-57 names no concrete second destination — it
  declines G1, G8 and N1 without proposing one — so it is not a contested
  assignment and does not appear there.
- **`upperBound` disclosures: ~~5~~ → 7.** Both new records disclose their own,
  and the two bounds run in opposite directions, which is why they are worth
  reading. R-57's is an over-count: acorn accepts `\u` and `\x`, which kali
  refuses outright, so a corpus carrying one would count a LOUD `E1004`
  divergence as this silent one. R-58's is an over-count too, but a small and
  exactly-bounded one — `{00: 1}`..`{07: 1}` match the pattern and agree with
  node, because a one-digit run reads the same in both radices (measured), while
  every longer run diverges (measured). R-58 is ALSO an **under**-count, which no
  other record in this catalogue is: its entry establishes that the same
  misreading fires outside key position (`console.log(042)` prints `42`, node
  `34`), and the matcher counts the key lane only. A reader must not treat R-58's
  0 as the frequency of the whole defect.
- **The instrument moved again, and the SHAs were re-pinned a second time.** Two
  matchers, two catalogue records, both frozen SHAs in
  `crates/kali_blast_radius/src/manifest_tests.rs`, and a regenerated
  `counts.json`. Per **spec §4.3** that is its own commit, landed ahead of this
  one; the counts diff adds exactly two entries and leaves every other entry's
  every field byte-identical.
- **ONE PROVENANCE CELL MOVED THAT IS NOT ABOUT THESE TWO ENTRIES, AND IT IS A
  READING RATHER THAN A DRIFT.** §1.4's `node` cell now says **`v26.8.1`** where
  it said `v26.7.0`, because that is what `node --version` prints on the machine
  this regeneration ran on. Every oracle transcript in the two new entries and
  their four cases was taken against that version and cites it; the older entries
  keep `v26.7.0` because that is what *their* readings were taken against, and
  neither number is retro-fitted onto the other. No count depends on the node
  version — the matchers run on acorn, still pinned at `8.18.0`, over a corpus
  whose hash is unchanged — but it is published provenance and it is disclosed
  here, in `manifest_tests.rs`, and in the apparatus commit's message.

**What this amendment does NOT close.** The amendment above lists two divergences
found while retiring R-56 and not filed. **One of them is now filed: the
escape-sequence one is R-57.** The other — `.length` on a string read out of an
ARRAY ELEMENT — is still unfiled, still live, and still outside this document's
figures; R-57's own entry re-measured it at `dde0f083c0` (`["a"]`→`2` against
node's `1`, `["ab"]`→`2`/`2`, `["abc"]`→`2`/`3`, `["abcdefghij"]`→`2`/`10`) and
records it as explicitly not R-57. Two further divergences from the same project
remain unfiled at this commit and are the subject of the next task on this
branch. Until they are filed, the structural blind spot the R-55 note describes
applies to them as it did to these two: an unfiled defect has no §0.2 row, no
predicate record and no count, and no gate notices its absence.

**AMENDMENT 2026-09-08, at ~~`35e9ef4ef6`~~ `f9e51b1dd4` — a FIFTH regeneration,
the second driven by a filing decision, and THE FIRST SINCE THE FREEZE IN WHICH A
NEW ENTRY MOVED A BAND.** (Corrected with the one above: `35e9ef4ef6` is the tree
R-59's and R-60's BEHAVIOUR was measured on, which is a different label from the
commit the generator ran at. This regeneration's provenance row records
`f9e51b1dd4`, and it landed in `02297ca6c2`. The heading convention is the
generator's own commit, which is what the sixth amendment below names.)
Written as a sibling of the amendment above rather than appended to it,
which is this section's own convention: one amendment per regeneration, each
naming its ordinal and its commit. The register-property-key-followups branch
filed **R-59** (§2, Tier 2 — a computed member index that is not a literal is
fabricated into a property name, so `o[i]` reads the property named `i`) and
**R-60** (§2, Tier 2 — a PRESENT property on an `Object.fromEntries` object reads
`0`, in a run where `Object.hasOwn` answers `true` about the same property).
Nothing in the compiler moved here either: both divergences were live at the
previous regeneration and are live now. **With these two, all four of the
divergences the amendment above and the one before it record as "found ... and
NOT filed" are filed**, which closes the thread those two amendments left open.
Read out of the regenerated §2 and §3, not predicted:

- **Ranked entries: ~~29~~ → 31**, and **§2's clusters ~~18~~ → 19 — a smaller
  jump than the last two entries produced, and that is the finding.** R-59 takes
  a new singleton, `R-59 (unclustered)`; **R-60 joins the EXISTING G6** and adds
  no cluster. It is the first entry this branch filed that joins a §3 cluster
  rather than declining one, and it joins by running the experiment G6's own
  §3 bullet asks for: *"call any other plausible-but-absent builtin and observe
  whether it yields `0` or `E3100`."* The answer, measured, is **both** —
  `Object.fromEntries` reaches codegen unresolved and is lowered through the
  zero-placeholder call fallback, which pushes an `E3100` **warning** that
  `kali run` never shows. G6's members are now R-15, R-24, R-25 and R-60.
- **A BAND MOVED, ON THE RAW AXIS, AND IT IS THE FIRST TIME A NEW DEFECT HAS DONE
  THAT IN THIS SERIES.** R-59 measures **raw 302 / reachable 45** — the first
  nonzero count added to this document since the freeze. Its cluster enters raw
  **band 2** (with G2), which pushes N1 out of band 2 and shifts every band below
  it down one: G8 3→4, G7 4→5, R-09 5→6, R-26 6→7, G6 and R-34 7→8, R-47 8→9, and
  the zero block 9→10. **The raw axis now has ten bands where it had nine.** No
  cluster changed its neighbours' *counts*; what changed is that a new cluster was
  inserted between two existing ones, which is what a band structure is for.
- **On the reachable axis no band boundary moved, and R-59 arrives in a TIE whose
  reason this amendment got wrong the first time.** `R-59 (unclustered)` enters
  reachable **band 2** at **45**, alongside `G3` at **45** — and G3's 45 *is*
  R-13's 45. This bullet originally explained the tie by saying the two clusters
  *"count the same sites"* because *"R-59's matcher is a strict subset of R-13's
  shape"*. ~~That~~ **The subset half is false, and the instrument disproves it in
  one line** (review round 1): `var o={}; o[true]; o[null]; o[/x/]; o[1n];`
  counts **0** under `computedMemberNonLiteralKey` and **4** under
  `computedMemberFabricatedPropertyName`, while
  `var o={1:"one"}; o[(1)]; o[(0,1)]; o[+1]; o[-1];` counts **3** under the first
  and **0** under the second. R-13's shape is
  `computed && property.type !== "Literal"`; R-59's asks whether
  `expression_to_property_name` can READ the index, and a boolean, `null`, a
  BigInt and a regex are all `Literal` nodes it cannot read. **The two overlap and
  neither contains the other**, and both directions are counted correctly —
  `o[true]`, `o[null]` and `o[1n]` each read the fabricated `index` property (`5`
  against node's `7`, measured at `35e9ef4ef6`, both scopes), and the four
  readable spellings each read the CORRECT name. **The corrected reason for the
  tie is that the corpus contains NEITHER separating family**, which is a fact
  about this corpus and not about the two shapes. The figures did not move; the
  explanation did, and the correction is recorded here rather than substituted,
  because a number published with a wrong reason is the failure this section
  exists to make visible. ~~Bands 3 to 6 are byte-identical~~ — **bands 3, 4 and
  6 are byte-identical; band 5's only change is G6's member list gaining R-60**
  (corrected 2026-09-08 in final review, by diffing this regeneration's §2.2 as
  it stood at `6f0df2c3db` against the fourth's as it stood at `35e9ef4ef6`: the
  whole reachable-axis diff is two
  lines, R-59's new band-2 row and G6's members going `R-15, R-24, R-25` →
  `R-15, R-24, R-25, R-60`). R-60 lands in reachable band 5 with the other zeros.
  **Band 1 is byte-identical to the block above on both axes**, which is now the
  third consecutive regeneration in which that has held.
- **§3.1's arithmetic moved in the numerator for the first time.** Entries
  entering with a nonzero reachable count go **~~7~~ → 8** (R-59 = 45 joins R-06,
  R-08, R-10, R-13, R-14, R-30, R-31); "19 measure zero" becomes **20**; and
  "18 clusters ... on the evidence of 7 nonzero entries" becomes **19 clusters on
  the evidence of 8**. The last two amendments each had to report that the
  ranking's numeric input did not grow at all; this one does not, and that is the
  difference between filing two zero-frequency entries and filing one with a
  frequency behind it.
- **AND THE NUMERATOR'S GROWTH IS DOUBLE-COUNTED, WHICH THIS AMENDMENT SAYS
  RATHER THAN LETTING THE FIGURE STAND ALONE.** R-59's 302 raw and 45 reachable
  sites are, to the digit, sites `G3` already counts through R-13 — ~~**on this
  corpus the same sites**~~ **on this corpus a strict SUBSET of them after the
  sixth regeneration below, which took store targets out of R-59's matcher; the
  double-count is partial, not total, and the warning not to add the two counts
  survives it** — in two clusters, summed twice by `aggregate`. (The
  qualifier is load-bearing after the correction above: the two matchers select
  the same set *here* because neither separating family occurs here, not because
  either shape contains the other. Wherever one did occur, the sets would differ
  and the double-count would be partial rather than total.) That is the same
  over-attribution `clusters.json`'s WHOLE-ENTRY RULE discloses for R-23 and
  R-28, arriving by a new route, and it is disclosed in `clusters.json`'s `note`
  and in `count.mjs`'s `UPPER_BOUNDS` as well as here. **A reader must not add
  R-13's count to R-59's.** It is not resolved by moving either entry into the
  other's cluster: R-59's own §2 entry establishes that R-13's **G3** assignment
  rested on a mechanism hypothesis — *an admit-list falling through to a
  default-`0` read* — that R-59 disproves by measurement (adding one property to
  R-13's own repro object turns its `0` into `99`), and re-clustering an existing
  entry on that evidence is a decision a filing task declines to take. The
  evidence is recorded so a later reader can take it.
- **§2.4's contested assignments: ~~10 of 29~~ → 12 of 31**, and both new entries
  enter, which is a first — the last amendment added one of two. R-59 names
  **G1** as the destination it declines (on G1's own traced mechanism: a token
  stream contract, and `expression_to_property_name` consumes no tokens), and
  R-60 names **G2** (declined because G2's members are all user function values
  and a missing builtin is what G6 holds). Both swaps price **`unchanged` on both
  axes**.
- **`upperBound` disclosures: ~~7~~ → 8 disclosed in the record, and ~~3~~ → 4
  NOT disclosed.** R-59's is disclosed and is an over-count with a measured
  boundary: a receiver allocated with `new Array(n)` reaches a runtime-index lane
  that evaluates the member node's structured index instead of the fabricated
  name and **agrees with node** (`0 2 4` on both engines), while the same loop
  over an array LITERAL prints `0 0 0` against node's `5 6 7`. Both spellings are
  counted; one of them is not this defect. R-60's is **not** disclosed in its
  record, and it runs in **both** directions — the second record in this
  catalogue to do so, after R-58. Over: every reading behind it is the default
  Fast build mode and the `--release` lane was not measured, because there is no
  runner for a built artifact on this machine. Under: the fabricated `0` is what
  an unresolvable static member read emits generally, and `Object.fromEntries` is
  one producer of one; the rest of that family — R-21's lane, and the `o[1]` read
  inside R-59's own repro — is counted nowhere.
- **The instrument moved again, and the SHAs were re-pinned a THIRD time.** Two
  matchers, two catalogue records, both frozen SHAs in
  `crates/kali_blast_radius/src/manifest_tests.rs`, and a regenerated
  `counts.json`. Per **spec §4.3** that is its own commit, landed ahead of this
  one; `accepts.mjs` was re-run first and wrote a byte-identical `accepts.json`,
  and the `counts.json` diff contains **no deletions at all** — it adds two
  entries and moves no other field, including `nodeVersion`, which stays
  `v26.8.1`.
- **NO PROVENANCE CELL MOVED THIS TIME except the HEAD row**, which moves on every
  regeneration by construction. `node` stays `v26.8.1`, `acorn` stays `8.18.0`,
  the corpus hash stays `ca6f5333…`, and §0.2's measured-at cell stays
  `62b11a78c3`. The one thing worth noting about provenance is a *distinction* the
  last amendment introduced and this one keeps: R-59's and R-60's measurements
  were taken on a binary built at **`35e9ef4ef6`**, not at `dde0f083c0`, because
  that is this branch's documentation-only tip and is the tree the binary was
  actually built from. The two commits differ only in documentation, so no
  measured behaviour can differ between them — but a reading is labelled with the
  tree it was taken on, not with the tree it would also have held on.

**What this amendment closes, and the one thing it does not.** The thread the two
amendments above left open is closed: all four divergences PR #34 measured and
deliberately did not file are now §2 Tier-2 entries — R-57, R-58, R-59, R-60 —
each with a §0.2 row, an oracle scope pair, a predicate record and a count. **The
`.length`-on-an-array-element divergence is still unfiled and still live**, and it
is a different lineage: it was found while retiring R-56, R-57's entry re-measured
it and records it as explicitly not R-57, and R-59's entry meets it again from the
other side (`o[1]` reading `0` where node reads `undefined`). Until it is filed the
structural blind spot the R-55 note describes applies to it: an unfiled defect has
no §0.2 row, no predicate record and no count, and no gate notices its absence.

**AMENDMENT 2026-09-08, at `07ad2e6447` — a SIXTH regeneration, the first driven
by a CORRECTION TO A MATCHER rather than by a filing, and the first in this
series in which a published figure went DOWN.** A sibling of the amendment above,
per this section's one-amendment-per-regeneration convention. Nothing in the
compiler moved, no entry was filed, and no cluster changed: what moved is that
`computedMemberFabricatedPropertyName` was counting the WRITE half of a READ-lane
entry. The final whole-branch review re-derived the split, and R-13's own record
had already published it: of R-59's raw 302, **67 were assignment or update
TARGETS**; of its reachable 45, **18 were — 40% of the headline**. R-59's entry
measures that a store does not fabricate (`o[i] = 8` leaves `o.i` at `7` and
`o.index` at `9` on both engines, exit 0, both scopes; `o[i]++` is refused
LOUDLY with `error[E5506]` at exit 1 where node prints `7` and `9`), so those
sites were never this defect. The matcher now excludes them, as R-60's already
did. Read out of the regenerated §2 and §3, not predicted:

- **Ranked entries stay 31 and clusters stay 19.** No entry entered or left, no
  assignment moved, and `clusters.json` gained nothing. This is the first
  regeneration in the series whose whole cause is a number.
- **R-59: raw ~~302~~ → 235, reachable ~~45~~ → 27** (anchor ~~47/43~~ → 27/25,
  extension ~~255/2~~ → 208/2). The four deltas are exactly R-13's record's own
  `upperBound.breakdown` storeTarget figures — raw 67, reachable 18, anchor
  20/18, extension 47/0 — which is the cross-check that the exclusion removed
  store targets and nothing else.
- **A BAND MOVED ON THE REACHABLE AXIS, AND IT IS THE FIRST TIME A BAND HAS MOVED
  BECAUSE A COUNT SHRANK.** The reachable axis goes **~~6~~ → 7 bands**. At the
  fifth regeneration `R-59 (unclustered)` sat in band 2 **tied with G3 at 45**;
  at 27 it can no longer tie, so band 2 holds G3 alone, R-59 becomes a band of
  its own at **band 3**, and every band below shifts down one: N1 3→4, G7 4→5,
  the zero block (G6, R-09, R-26, R-27, R-28, R-47, R-57, R-58) 5→6, R-34 6→7.
  **The tie the amendment above spent a paragraph explaining is gone**, and it is
  worth saying why it existed: it was never about the two shapes, and after this
  correction it is not even about the corpus — R-13 counts store targets and
  R-59 does not, so the two cannot tie here at all.
- **The raw axis did NOT move: ten bands, no boundary changed, and the whole
  §2.3 diff is one number.** `R-59 (unclustered)` stays in raw band 2 with G2,
  at 235 instead of 302. The band structure the fifth regeneration created by
  inserting R-59 between two existing clusters survives intact.
- **Band 1 is byte-identical on both axes** — the FOURTH consecutive regeneration
  in which that has held.
- **§3.1's arithmetic did not move at all**, which is the one figure a reader
  might expect to. Entries entering with a nonzero reachable count stay **8**
  (R-59 = 27 instead of 45, still nonzero); "20 measure zero" stays 20; "19
  clusters on the evidence of 8" stays. A count shrinking by 40% changed a band
  and left the denominator argument untouched.
- **§2.4 stays 12 of 31, but TWO COUNTERFACTUAL CELLS CHANGED**, and they are the
  only cells outside R-59's own rows that this regeneration touched. In the fifth
  regeneration, moving **R-21** or **R-23** to its alternate cluster made the
  reachable band 1 gain **G3 *and* `R-59 (unclustered)`**, because R-59 was tied
  with G3 at 45 and rose with it. At 27 it no longer does: both rows now gain
  **G3 alone**. Both swaps still price the same verdicts overall.
- **`upperBound` disclosures are unchanged at 8 disclosed / 4 not**, and R-59's
  record still discloses its two over-counts (the `new Array(n)` receiver, and
  the LOUD regex spelling). What the record gained is a *narrowing*, not a new
  caveat: it now says it counts a member **READ**.
- **THE R-59/R-13 RELATIONSHIP WAS RE-MEASURED, NOT RE-REASONED, AND IT IS NO
  LONGER AN IDENTITY.** The fifth regeneration recorded that the two matchers
  print the same four numbers here; they no longer do, and the new relationship
  was measured over the frozen corpus file by file: of the **51** files with a
  nonzero count under either matcher, **30** now differ (**8** of the **14**
  reachable ones), and R-59's count exceeds R-13's in **ZERO** files. So on this
  corpus R-59's site set is now a strict subset of R-13's. **That is still not
  containment between the two shapes** — the withdrawal of `4ed6e9ede8` stands,
  and both separating families it measured are still real and still absent from
  this corpus. Store targets are simply the one separating family the corpus does
  exercise, and they run R-13's way. The double-count bullet above is amended in
  place accordingly: it is now a partial overlap, and **a reader still must not
  add R-13's count to R-59's**.
- **NO PROVENANCE CELL MOVED except the HEAD row** (`f9e51b1dd4` →
  `07ad2e6447`). `node` stays `v26.8.1`, `acorn` stays `8.18.0`, the corpus hash
  stays `ca6f5333…`, and §0.2's measured-at cell stays `62b11a78c3`. `accepts.mjs`
  was re-run first and wrote a byte-identical `accepts.json` (anchor 126/137,
  extension 1/40); `counts.json`'s whole diff is R-59's five figures plus its
  `note`. Both frozen SHAs were re-pinned — the **FIFTH** movement of those
  constants — with the rationale in `manifest_tests.rs` extended rather than
  rewritten.

**What this amendment is really about.** The fifth regeneration published R-59's
45 as *"the first nonzero count added to this document since the freeze"* and
built a band around it. 40% of it was a site class the entry's own body says is
not the defect — and the evidence was already in this repository, in R-13's
`upperBound.breakdown`, one field away from the number being published. Nothing
gated it: `check_completeness` checks that every record has a matcher, and the
freeze checks that no matcher changes without a re-pin, but no gate asks whether
a matcher counts what its record says. That question is answered by review, and
this amendment is what answering it late looks like.

**AMENDMENT 2026-09-09, at `71b5f42f6c` — a SEVENTH regeneration, and the first
in which the ACCEPT SET moved.** The computed-member-static-name project
(`docs/superpowers/specs/2026-09-08-computed-member-static-name-design.md`) made
`kali` REFUSE a computed member access whose index it cannot read statically.
That closed **R-13** (FIXED, both lanes) and **R-59** (FAIL_CLOSED, one lane),
moved **R-12** to FAIL_CLOSED without closing it, and — for the first time since
this document existed — changed which corpus programs kali accepts, so
`accepts.json` and `counts.json` were re-measured against the new binary in
their own `measure(blast-radius)` commit before the ranking was regenerated. The
instrument did not move: `predicates.json`, `matchers.mjs` and both frozen SHAs
in `crates/kali_blast_radius/src/manifest_tests.rs` are untouched, so a
regression re-lights the same numbers. Every figure below is **read out of the
regenerated §2–§5 and out of the `counts.json` diff**, not predicted; §6.6 item
4's instruction is *re-run, do not re-read*.

- **THE ACCEPT SET MOVED, and that is the new thing here.** §5: anchor
  **126/137 (92.0%) → 126/137 (92.0%)**, unchanged; extension **1/40 (2.5%) →
  0/40 (0.0%)**; pooled reachable **127 → 126**. Exactly one program left the
  accept set — `extension/unit_conversions.js`, which reads `PREFIXES[prefix]`
  off a **parameter** key and now takes the shared computed-member `E5506` in
  both twins — and it was the extension stratum's last accepted program. The
  reachable axis is now computed over **anchor programs only**: §2.2's own
  sentence goes from "126 of those 127 are ANCHOR" to "126 of those 126", and
  "1 of the 40 programs written to do a job … is reachable" to "0 of the 40".
  Every previous amendment moved verdicts; this one moved the denominator.
- **Reachable counts changed on EIGHT entries**, all through that one program:
  R-04 ~~5~~ → **0**, R-07 ~~82~~ → **78**, R-08 ~~15~~ → **14**,
  R-13 ~~45~~ → **43**, R-14 ~~11~~ → **7**, R-32 ~~5~~ → **0**,
  R-33 ~~1~~ → **0**, R-59 ~~27~~ → **25**. Three of them (R-04, R-32, R-33)
  fall to a zero of a different KIND and `count.mjs` says so: their `zero` field
  goes from `null` to **`present-but-unreachable`**. No raw count moved at all,
  because the corpus did not change — only its accepted subset did.
- **Ranked entries: ~~31~~ → 28**, and **§2's clusters ~~19~~ → 17.** Two
  clusters left: `R-59 (unclustered)`, whose singleton definition goes with its
  assignment on the R-56 precedent, and **G3**, which is the first
  NON-singleton cluster ever removed from this ranking. R-12 and R-13 were G3's
  only two members and **both** left in this regeneration, so
  `crates/kali_blast_radius/src/ranking.rs:326` ("an empty cluster ranks
  nothing") forced the definition out too. G3 survives as a register §3 group;
  what left is its row in the ranking, which ranks the SILENT class only.
- **G3's frequency, before → after: reachable ~~45~~ → gone; raw ~~305~~ →
  gone.** It was reachable band 2 (alone) and raw band 1.
- **BANDS MOVED ON BOTH AXES, and the raw band 1 changed membership.** Checked by
  differencing the band blocks of the pre- and post-splice documents, not by eye.
  *Reachable*: **band 1 is byte-identical**, six rows, unchanged; below it,
  bands ~~7~~ → **5** — G3 (45) left band 2, `R-59 (unclustered)` (27) left band
  3, and **N1 — escape/provenance loss rose from band 4 to band 2** while its own
  frequency FELL from ~~11~~ to **7** (R-14's 11 → 7). A cluster rising two bands
  while losing frequency is what removing everything above it looks like.
  *Raw*: bands ~~10~~ → **8**, and **band 1 lost G3 (305) and gained N1 (99)** —
  the first time a countable cluster has entered or left a band 1 in this
  document. `G8` rose from raw band 4 to raw band 2 (79, unchanged frequency),
  and G2 keeps raw band 2 with it. The **countable-only** raw band 1 goes
  ~~"R-52 (tier 1, 5); G3 (tier 2, 305)"~~ → **"R-52 (tier 1, 5); N1 (tier 2,
  99)"**; the countable-only reachable band 1 is unchanged.
- **§2.4's contested assignments: ~~12~~ → 11**, and **THREE surviving rows
  changed verdict** — the most §2.4 movement any regeneration has produced.
  R-59's own row left with it. Of what remains: **R-08** goes ~~"gains G3; loses
  G8"~~ → **`unchanged`** on the reachable axis, **R-21** goes ~~"gains G3;
  loses G4"~~ → **`unchanged`** on the reachable axis (its raw verdict is
  unchanged and still moves band 1), and **R-23** goes ~~"gains G3"~~ →
  **"gains N1"**. All three moved for one reason: their alternate destination
  was G3, and G3 is no longer a cluster. **This exactly reverses the 2026-08-16
  amendment's most alarming finding**, which recorded R-08 becoming load-bearing
  when G8 fell from 65 to 59; the swap is priced `unchanged` again, not because
  the frontier got robust but because the cluster it moved into stopped
  existing. §6.3's point stands, with the sign flipped.
- **§3.1: entries removed by the SILENT filter ~~15~~ → 18**, of which the *not
  damage* group goes ~~13~~ → **16** — R-12 and R-59 join at FAIL_CLOSED, R-13
  at FIXED. "Only ~~4 of the 15~~ **3 of the 18** removed entries have a nonzero
  reachable count at all" and the list is now **R-07 (78), R-13 (43), R-59
  (25)** where it was R-04 (5), R-07 (82), R-32 (5), R-33 (1) — a complete
  turnover of that list in one regeneration, three entries falling to zero as
  the accept set shrank and two arriving with the largest counts they ever
  carried.
- **Entries entering with a nonzero reachable count: ~~8~~ → 6** (R-06 = 1,
  R-08 = 14, R-10 = 1, R-14 = 7, R-30 = 57, R-31 = 2); the zeros go ~~20~~ → 19
  and the no-count entries stay at 3. §3.1's closing sentence now reads "the
  bands below separate **17** clusters on the evidence of **6** nonzero
  entries", from ~~19~~ and ~~8~~. **That is the thinnest numeric evidence this
  ranking has ever rested on**, and it is the figure to quote when someone asks
  how much the bands are worth.
- **`upperBound` disclosures: still 8 disclosed + 4 undisclosed.** Two gained
  the generator's `(not in the ranking)` annotation — **R-59** among the
  disclosed and **R-13** among the four this measurement found — joining R-56
  and R-07. The count is derived from `counts.json` rather than from the SILENT
  filter, so a retired entry's disclosure survives its departure, exactly as
  R-56's did.

**Whether the figures matched what was predicted, stated plainly because the
whole point of this section is that it must not be assumed.** The project's spec
§7 predicted three things, marked in the spec itself as "Expected, not
asserted": that **"G3 keeps R-12 alone and drops from 45 to R-12's own count"**,
that **"R-59's singleton cluster is removed"**, and that **"R-13's
contested-assignment row leaves §2.4"**.

- The second held exactly.
- **The first was wrong, and it is the finding of this regeneration.** G3 did
  not keep R-12: the gate
  `every_zero_two_row_is_the_class_set_its_live_cases_assert` named **three**
  entries, not two — R-12's oracle cases had flipped to `fail_closed` as well,
  because the project's bracket-store choke point refuses an aliased array store
  — so R-12's §0.2 row was re-derived in the same pass and G3 lost both members
  and its definition. The spec's prediction was about which rows would move, and
  it under-counted by one.
- **The third was wrong in a harmless way**: R-13 has no `alternateCluster` and
  never had a §2.4 row. The row that left §2.4 is **R-59's**. What the
  prediction missed is the interesting part — three OTHER entries' §2.4 rows
  changed verdict, which no prediction anticipated.
- **Not predicted at all**: the accept-set movement's size (one program, eight
  entries' reachable counts), the raw band 1 membership change, and N1 rising
  two bands on the reachable axis while its frequency fell.

**R-12 IS NOT CLOSED, and this amendment is not the place a reader should learn
otherwise.** It leaves the SILENT filter because its class moved from SILENT to
FAIL_CLOSED, not because it was fixed: `const a=[1,2]; const b=a; b[0]=7;
console.log("b0="+b[0])` still does not print node's `b0=7` — it exits 1 with
the shared computed-member `E5506`. That is the R-32 precedent this document
already records ("leaving is a statement about the dangerous class, not about
being fixed"), and R-12's §0.2 row says so at length. A second thing that row
records and this one repeats: `kali check` exits **0** on that program while
`kali run` refuses, a spec §8 twin disagreement the oracle harness cannot see
because it observes `run` only.

**One sentence inside the generated region has gone partly stale, and it is NOT
edited here.** §2.2's and §2.3's band-1 footnote — "§2.4 … finds two that move a
band 1: R-21 (both axes) and R-23" — is illustrative prose hard-coded in
`crates/kali_blast_radius/src/ranking.rs:520`, not a computed figure. After this
regeneration R-21 moves the **raw** band 1 only, not both, and R-23 moves the
reachable band 1 by gaining **N1** rather than G3. The count of two is still
right and both named entries still move a band 1, so the sentence is not false
in its claim, only in its parenthesis. Correcting it means editing the
generator, which is apparatus, and this task changed no apparatus — the same
decision, and the same disclosure, the 2026-08-16 retirement amendment made
about §3.4's illustrative prose. Recorded here so the omission reads as a
decision rather than an oversight.

**One divergence was found while measuring and is NOT filed as a register
entry.** `docs/superpowers/followups/member-length-renders-the-child-count.md`:
`.length` on a member-expression receiver renders a node's CHILD COUNT rather
than a length. **This project moved its computed half and left its dot half
silent** — `const o = {a:"xyz"}; o.a.length` still prints `1` at exit 0 where
node prints `3`, and `o["a"].length` still prints `2`, while the folded and
computed-callee spellings now refuse. It is filed as a follow-up rather than as
a §2 entry because a §2 entry is a dozen coordinated edits (a predicate, a
matcher, a counts re-freeze, an oracle pair, a clusters row) and that is a task
of its own. This is the same disclosure R-59's own §2 entry makes about the
leads it measured and declined to file, and the same one the R-56 retirement
amendment above makes about the two divergences it found.

**Five further findings this project measured and did not fix are filed
elsewhere**, listed here so the ranking's reader can find them: the
release-mode optimizer defect that destroys array identity
(`docs/superpowers/followups/release-mode-optimizer-inlines-an-allocating-initializer.md`,
the most consequential of the six and wider than this project's surface), the
`process.argv` `[2]`-vs-`["2"]` divergence, a BigInt rendering divergence on the
newly-live bracket-store lane, a `Uint8Array` recognizer tripwire, and the
batch-3 case generator's ability to silently revert two re-pins
(`docs/superpowers/followups/computed-member-static-name-discovered-defects.md`).
None of the six has a §0.2 row, so none of them is in the population this
document ranks — which is the same shortfall §6's 2026-08-16 note records about
R-55, and it is now five entries wider.

**AMENDMENT 2026-09-11 — an EIGHTH regeneration, filing R-63.** The
length-fails-closed project (`docs/superpowers/specs/2026-09-11-length-fails-closed-design.md`)
filed **R-63** — `.length` renders a node's child count, or `0`, for a receiver
with no length lane — as a SILENT Tier 2 entry in **G6**, in a commit of its own,
before the fix that retires it. Its record is countable
(`lengthReadOnUnprovenReceiver`): raw 35 / reachable 0, every site in the
extension stratum, so it enters §3 as `present-but-unreachable` and adds nothing
to G6's reachable figure. G6's §2 row is now `| G6 — unresolved or unimplemented
builtins fold to a default instead of failing closed | 2 | 0 | R-15, R-24, R-25,
R-60, R-63 |` (was `| G6 — unresolved or unimplemented builtins fold to a
default instead of failing closed | 2 | 0 | R-15, R-24, R-25, R-60 |`). **On the
raw axis (§2.3), G6 moves from Band 6 to Band 3** — `| G6 — unresolved or
unimplemented builtins fold to a default instead of failing closed | 2 | 38 |
R-15, R-24, R-25, R-60, R-63 |` (was `| G6 — unresolved or unimplemented
builtins fold to a default instead of failing closed | 2 | 3 | R-15, R-24, R-25,
R-60 |` in what was then Band 6) — frequency 3 → 38, R-63's raw 35 accounting
for the jump and by far the largest frequency swing anywhere in the regenerated
tables. That move displaces G7 from Band 3 to Band 4, R-09 from Band 4 to Band
5, and R-26 from Band 5 to Band 6, and merges R-34 into Band 7 alongside R-47;
none of those four entries' own frequency changed, only the band the peeling
computation assigns them. `counts.json` was regenerated, which also moved
`nodeVersion` from `v26.8.1` to `v26.8.2` and added R-61's null record; no other
matcher body changed. **The accept set was re-measured at the baseline binary
`152fdd5364`**, so that the next regeneration's movement belongs to this project
alone: it did not move. Every figure above is read out of
the regenerated §2–§5 and the `counts.json` diff.

**AMENDMENT 2026-09-11 — a NINTH regeneration, retiring R-63.** The
length-fails-closed project's fix landed: `.length`'s floor refuses and
`render_length`'s fallbacks decline. **R-63** moved to FAIL_CLOSED in both scopes
and left the SILENT filter, and with it G6's assignment. **R-15** gained a
FAIL_CLOSED lane (`r15`, whose repro reads `.length` first) and kept a SILENT one
(`r15e`, the element read alone), so it stays in G6 and in the ranking.
The accept set did not move. `accepts.json` and `counts.json` were regenerated
against the fixed binary and came out byte-identical (anchor 126/137 accepted,
extension 0/40), so no `reachable` figure moved either; R-63's own record still
reads raw 35 / reachable 0.
Nothing moved in `predicates.json`, `matchers.mjs` or the frozen SHAs; a
regression re-lights the same numbers. G6's §2 row is now `| G6 — unresolved or
unimplemented builtins fold to a default instead of failing closed | 2 | 0 |
R-15, R-24, R-25, R-60 |` (was `| G6 — unresolved or unimplemented builtins fold
to a default instead of failing closed | 2 | 0 | R-15, R-24, R-25, R-60, R-63 |`),
still in Band 4: no band moved on the reachable axis (§2.2). **On the raw axis
(§2.3), G6 moves from Band 3 back to Band 6** — `| G6 — unresolved or
unimplemented builtins fold to a default instead of failing closed | 2 | 3 |
R-15, R-24, R-25, R-60 |` (was `| G6 — unresolved or unimplemented builtins fold
to a default instead of failing closed | 2 | 38 | R-15, R-24, R-25, R-60, R-63 |`
in Band 3) — frequency 38 → 3, R-63's raw 35 leaving with it. That move returns
G7 from Band 4 to Band 3, R-09 from Band 5 to Band 4, R-26 from Band 6 to Band
5, and R-34 from Band 7 to Band 6 alongside G6, leaving R-47 alone in Band 7;
none of those clusters' own frequency changed. Elsewhere in §2–§5: §2.1 drops
R-63's assignment row; §2.4 prices 11 of the 28 ranked entries (was 12 of the
29), losing R-63's contested row; §3's per-entry table drops R-63, and R-15's
lanes cell reads `FAIL_CLOSED / SILENT` (was `SILENT`); §3.1's SILENT filter
removes 19 entries (was 18), 17 of them as not damage (was 16), and lists R-63
there at raw 35 / reachable 0 / `FAIL_CLOSED`, while of the 28 entries that enter
(was 29) 19 measure zero (was 20) and the six nonzero reachable counts are
unchanged; §3.3 marks R-63 `(not in the ranking)`; §3.4 counts 10 of the 28
ranked entries with a non-SILENT lane (was 9 of the 29), adding R-15 (FAIL_CLOSED
/ SILENT). §4 and §5 did not move. Every figure above is read out of the
regenerated §2–§5 and the `counts.json` and `accepts.json` diffs.

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
**zero** — G2 (R-51) and R-52 — alongside G8 at ~~65~~ **59** (amended 2026-08-16;
see the amendment at the head of §6). That is not a glitch. Tier 1
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
cluster — and it is G8's ~~tier-2-with-65~~ **tier-2-with-59** combination
(amended 2026-08-16) that dominates G3's tier-2-with-45. Move R-23 to G4, as the
register's own second reading allows, and G8 becomes tier 4, stops dominating,
and **G3 enters band 1**. The margin narrowed on 2026-08-16 — 59 against 45,
where it was 65 against 45 — which is a second reason to read this subsection
before citing band 1: the dominance survived, and §2.4 now shows the R-08 swap
changing the reachable band 1 where it previously changed nothing.

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

- **It does not retire anything.** §3.4 lists the ~~eleven~~ **nine** (amended
  2026-08-16) ranked entries with a non-SILENT lane. A lane is not an entry:
  R-47's and R-53's FIXED lanes are the `const` controls those entries declare
  for themselves, ~~R-30's two FIXED lanes are its `const`-scalar lane *and* its
  concat/template sinks~~ — **amended 2026-08-16: R-30 now has six FIXED
  case-lanes, four of them declared controls and two of them lanes that genuinely
  moved (its `const`-object-field lane, and the taint-reaching `String()`-result
  lane at the multi-argument sink); its plain `var`-binding lane is still SILENT
  and is what keeps it open** — R-08's `===` half
  fails closed while its `??` half is **still SILENT**, and R-49 — outside the
  ranking entirely — fails closed by *R-35's* switch allowlist rather than by any
  gate of its own.
  - **This bullet survived an entry actually retiring, and the distinction it
    draws is why.** R-33 retired on 2026-08-16. Nothing in this document retired
    it: its two oracle cases were re-measured against node, both came back FIXED,
    and §0.2 was re-derived from them. The ranking then dropped it mechanically,
    because an entry with no SILENT lane leaves the SILENT filter. §3.4 now states
    the structural consequence — a retired entry can never appear in that list,
    because it is filtered out before the list is built — so a reader cannot infer
    "not retired" from absence there either. R-32 left the same way and is **not**
    retired; leaving is a statement about the dangerous class, not about being
    fixed.
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
