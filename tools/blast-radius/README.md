# The blast-radius counter and accept table

Two instruments over the frozen corpus in `corpus/`, deliberately independent
of the register oracle:

- **`accepts.mjs`** — reachability. For every corpus program, does
  `kali check` exit 0? Writes `accepts.json`.
- **`count.mjs`** — frequency. For every countable predicate in
  `predicates.json`, how often does its triggering construct occur? Writes
  `counts.json`, with each count reported both raw and gated on reachability.

## Run them, in this order

```bash
cargo build -p kali_cli          # from the repo root
cd tools/blast-radius
npm ci                           # exact versions from package-lock.json
node --test                      # the matcher tests
node accepts.mjs                 # writes accepts.json
node count.mjs                   # reads accepts.json, writes counts.json
```

**Order matters.** `count.mjs` consumes `accepts.json` to compute the reachable
column, and refuses to run if it is missing or was generated against a
different `corpus_hash`.

`accepts.json` and `counts.json` are **committed outputs**. They are the
published numbers; the ranking cites them, and a reader can diff them against a
re-run.

## Why acorn and not `kali_parser`

Counting the constructs kali miscompiles with kali's own parser is a confounded
instrument — `sweep-common.md` rule 3 exists to prevent exactly this, and R-49
is the proof it is not hypothetical: `parse_switch_statement` silently
reparented every post-switch statement for weeks with the suite green. A parser
with a defect in it cannot be the instrument that measures how often that
defect's construct occurs.

`acorn` and `acorn-walk` are pinned to exact versions in `package.json`, with
`package-lock.json` committed, so the same source always parses to the same
tree. `matchers.mjs` imports `acorn` and builds one analysis pass of its own
rather than using `acorn-walk`'s visitors: several predicates are stated in
terms of a node's parent, its enclosing scope, or what a name is bound to, and
`walk.simple` hands a visitor neither. `acorn-walk` stays pinned because the
tool's dependency set is part of what the published numbers are reproducible
from.

## What each file is

| file | what it is |
|---|---|
| `predicates.json` | **Frozen.** 41 records, 37 countable with matcher names, 4 uncountable. Written and reviewed in Task 2; not editable here. |
| `corpus/` | **Frozen.** 177 programs in two strata, `corpus_hash` in `corpus/manifest.json`. See `corpus/README.md`. |
| `matchers.mjs` | One matcher per countable record, each implementing what that record's `description` says. |
| `matchers.test.mjs` | A positive and a negative test per matcher, plus the two module gates. |
| `corpus.mjs` | Freeze verification and kali-binary resolution, shared by both tools. |
| `accepts.mjs` / `count.mjs` | The two tools. |
| `accepts.json` / `counts.json` | Committed outputs. |

## The four rules these tools enforce on themselves

1. **The kali binary is resolved, not guessed.** `KALI_BIN` if set, else
   `<target_directory>/debug/kali` read from `cargo metadata` — never a
   hardcoded `target/` path, which does not exist in every checkout. It is
   verified to exist, to be executable, and to answer `--version` **before**
   anything is measured. "kali ran and rejected the program" and "kali could
   not be run" are different outcomes: the second aborts, and is never recorded
   as `accepted: false`. A wrong path would otherwise mark every program
   unreachable and publish an all-zero reachable count that looks like data.
2. **Counts are reported per stratum as well as pooled.** The anchor is 131
   micro-snippets plus 6 real programs — 4.4% of anchor programs but 56.7% of
   anchor bytes — so a pooled count is dominated by the anchor's shape.
   `counts.json` carries `strata.anchor` and `strata.extension` beside the
   pooled `raw`/`reachable`. Accept **rates** are never pooled at all.
3. **The freeze is verified at measurement time.** Both tools recompute every
   corpus file's sha256 and the `corpus_hash` itself, check both directions
   against the manifest (no missing file, no untracked `.js` under the corpus
   root), and abort on any disagreement. The Rust side enforces this in tests;
   the tools that produce the published numbers enforce it themselves.
4. **An empty manifest refuses.** A corpus of nothing would make every rate
   `0/0` and every predicate score identically.

Plus two gates that make a wrong number loud rather than quiet:

- **The catalogue and the matcher module must agree in both directions.** A
  catalogue record naming a matcher that does not exist would contribute
  nothing silently; a matcher no record names would be counted for no entry.
  Either one aborts `count.mjs`.
- **A syntax error is thrown, never counted as zero.** A file that fails to
  parse would otherwise report "this construct does not appear here" — a
  measurement it did not make.

## Reading `counts.json` — what a number does and does not mean

`counts.json` is the authority for every figure below; it carries all of this as
data, so nothing here needs retyping downstream. This section says where to look
and why it matters.

### The reachable column is an anchor-snippet column

126 of the 127 reachable programs are **anchor** programs, and the anchor is 131
micro-snippets written to probe compiler behaviour plus 6 real CLBG programs. So
every reachable ranking is, in substance, a ranking over test snippets. The
extension stratum — the 40 programs written to *do jobs* — is accepted **1/40 =
2.5%**, so nearly everything it measures about real programs lands in the **raw**
column only. Always read a per-entry `strata` split before treating a reachable
figure as a frequency in real code. This is in `counts.json` under `population`.

The accept rate is also a finding in its own right, not a defect of the corpus:
curation was independent of acceptance, and kali was never run while the corpus
was written.

### Some counts are upper bounds — `entries[].upperBound`

Four records carry an explicit upper-bound clause naming what the AST cannot see
(**R-08**, **R-16**, **R-26**, **R-30**). Three more are upper bounds that their
records do **not** disclose, found while implementing them (**R-13**, **R-14**,
**R-07**). Each carries a `note` saying what it cannot see, and
`disclosedInRecord` says which kind it is.

The sharpest is **R-13**. Its record is "computed member access whose key
expression is not a literal", with no qualifying clause, so ordinary array
indexing `a[i]` counts — and array indexing demonstrably works. The register's
R-13 repro is an *object* read with a variable key. `upperBound.breakdown` gives
the split of the same 302 sites: 56 have an object-literal receiver, 45 an
array-like one, and **67 are store targets rather than reads**. Do not present
the total as "how often R-13's defect is triggered".

### Two counts rest on an interpretation — `entries[].alternateReading`

Where a record states its shape twice and the two statements disagree on this
corpus, both numbers are published, with which one the count uses and why.

| entry | published reading | alternate reading |
|---|---|---|
| **R-07** | main clause, "is not a literal" — **449 raw / 82 reachable** (ranks 1st) | dash-list read as exhaustive — **286 / 43** (ranks 3rd, behind R-30 and R-13) |
| **R-02** | complement clause — **2 / 0** | role list read as exhaustive — **0 / 0** |

R-07's is the consequential one: the reading decides the top of the ranking. The
disputed sites are `new` expressions, object literals and array literals —
forms named in neither the record's list nor the register's shape survey. The
main clause was chosen because it governs and the appositive illustrates, and
because the register bounds the damage the other way round ("a `const` bound to
a literal is correct"). It is still an interpretation, and it is published as one.

### Three kinds of zero — `entries[].zero`

`zeroKinds` in `counts.json` carries these definitions; the classification is
mechanical, not left to the reader.

- **`structurally-uncountable`** — the construct cannot appear in any conforming
  corpus program. **R-29 alone**: an assignment to a `const` is a runtime
  `TypeError`, so no program that runs clean can contain one. Not a frequency;
  must never be ranked against measured frequencies.
- **`unsampled`** — countable and legal, but absent from this corpus: **R-15,
  R-18, R-27, R-28, R-48**. An ordinary zero over this population, saying
  nothing about a larger one.
- **`present-but-unreachable`** — `raw > 0, reachable = 0`: the construct **does**
  occur, but every program carrying it is rejected by kali as a whole. Twenty
  entries, R-01 (18 raw / 0 reachable) among them. **This is the most
  misreadable of the three.** It does not mean the construct is rare, and it
  does not mean kali fails closed on that construct — the carrying program was
  usually rejected for an unrelated reason elsewhere in the file. Given the 2.5%
  extension accept rate, it is the common case.

### The population is a dialect, not JavaScript

The extension is written in the project's imperative-core dialect: no regex, no
destructuring, no template literals, no `??`, no class/Map/Set/async. A
frequency here is a frequency in *programs of that dialect*. `corpus/README.md`
records which counts that biases and in which direction (R-13, R-14, R-16 and
R-19 upward; R-08's `??` lane entirely unsampled).

## Known gap: the counter is not wired into CI

`node --test` in this directory is **not** run by `scripts/test-gate.sh` or
`.github/workflows/ci.yml`. Wiring it in would mean editing a file under this
project's do-not-modify constraint, so it is recorded here rather than worked
around. The matchers are therefore covered by their own tests but not by the
gate; a change to `matchers.mjs` that breaks them will not turn CI red.

This sits alongside the other open CI-lane gap,
`docs/superpowers/followups/test-binary-consolidation-determinism-lane.md`
(`scripts/check-determinism.sh` runs zero tests). Both need the same kind of
human decision: a change to a do-not-modify file.
