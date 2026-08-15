# Blast-Radius Ranking Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn "blast radius" into a reproducible measurement, re-measure every silent-miscompile register entry at current HEAD, and publish a banded ranking that names what to fix next.

**Architecture:** A new leaf crate `kali_blast_radius` holds the pure logic — register parsing, the predicate catalogue, the verdict classifier, and Pareto banding — so all of it is unit-testable without running a process. `kali_case_runner` gains a fourth step kind, `oracle`, that runs kali and node over one source and asserts the *derived verdict class*, making the register's verdict table regenerate instead of rot. A separate node tool under `tools/blast-radius/` counts triggering constructs over a frozen corpus with acorn. The two instruments never talk; they meet only in a final scoring step that is arithmetic over two JSON tables.

**Tech Stack:** Rust 2021 (workspace deps only), `libtest-mimic` via the existing case runner, node + `acorn` for the counter, JSON for every machine-read table.

**Spec:** `docs/superpowers/specs/2026-08-15-blast-radius-ranking-design.md`

## Global Constraints

- **Do-not-modify files.** `scripts/test-gate.sh`, `scripts/check-determinism.sh`, `mise.toml`, `.github/workflows/ci.yml`. Never edit these. If a task appears to need one, stop and record a follow-up in `docs/superpowers/followups/` instead.
- **No new external Rust crates.** Everything below is implementable with the workspace dependencies already in `Cargo.toml` (`serde`, `serde_json`, `sha2`, `toml`, `tempfile`, `libtest-mimic`). Process timeouts are implemented with threads, not a new crate.
- **Machine-read tables are JSON**, not TOML — node reads JSON natively, and Rust already has `serde_json`.
- **Freeze before counting.** The corpus manifest and the predicate catalogue are committed *before* the counter is run for record (spec §4.3). Never adjust either after seeing scores.
- **Never fabricate a frequency.** An entry with no syntactic predicate is `uncountable` with a written reason. No estimates, no "roughly" (spec §3.2).
- **No ran-nothing-green.** Every lane asserts a nonzero expected count. A missing fixture, a node that fails to launch, or a filter matching nothing is a failure, never a pass (spec §9).
- **Tier is not redefined.** Tier comes from the register's existing §2 section headers: 1 drops code/output, 2 wrong value, 3 wrong control flow, 4 rendering-only.
- **The register path** is `docs/superpowers/followups/kali-silent-miscompile-register.md` throughout.
- **The oracle is `node`**, resolved from `PATH` unless `KALI_ORACLE_NODE` is set. Its `--version` is recorded in every generated table.

---

## File Structure

**New crate — `crates/kali_blast_radius/`** (leaf; no kali deps, so nothing can cycle)

| File | Responsibility |
|---|---|
| `Cargo.toml` | package + `serde`, `serde_json`, `sha2` |
| `src/lib.rs` | module wiring and public re-exports |
| `src/register.rs` | parse `(id, tier, title)` out of the register's §2 markdown |
| `src/register_tests.rs` | |
| `src/catalogue.rs` | load `predicates.json`; completeness check against the register |
| `src/catalogue_tests.rs` | |
| `src/verdict.rs` | `Run`, `Verdict`, `classify`, `is_documented_code`, `runs_agree` |
| `src/verdict_tests.rs` | |
| `src/score.rs` | cluster aggregation, `dominates`, `band` |
| `src/score_tests.rs` | |

**Modified — `crates/kali_case_runner/`**

| File | Change |
|---|---|
| `Cargo.toml` | add `kali_blast_radius` |
| `src/model.rs` | `StepKind::Oracle`; `RawStep`/`Step` gain `register_entry`, `program`, `verdict`, `timeout_ms`; `finalize_step` applicability |
| `src/model_tests.rs` | parse + applicability tests |
| `src/steps.rs` | `run_oracle`, `capture_with_timeout` |
| `src/steps_tests.rs` | |

**New — `tools/blast-radius/`**

| File | Responsibility |
|---|---|
| `predicates.json` | the catalogue: one record per register entry |
| `corpus/manifest.json` | per-file sha256 + the corpus hash |
| `corpus/anchor/*.js`, `corpus/extension/*.js` | the frozen corpus |
| `package.json`, `package-lock.json` | acorn, pinned exact |
| `matchers.mjs` | one matcher per countable predicate |
| `count.mjs` | walk corpus, apply matchers, emit `counts.json` |
| `accepts.mjs` | run `kali check` per program, emit `accepts.json` |
| `matchers.test.mjs` | known-answer tests |
| `README.md` | how to re-run, and why it is not in CI |

**New — `crates/kali_cli/tests/cases/oracle/*.toml`** — the ~84 oracle cases.

**Modified docs** — `specs/15-errors.md` (the `E4xxx` gap), the register (§0.2 regenerated, §0.1 superseded), and `docs/superpowers/followups/blast-radius-ranking.md` (new).

---

## Task 1: Register parser

**Files:**
- Create: `crates/kali_blast_radius/Cargo.toml`
- Create: `crates/kali_blast_radius/src/lib.rs`
- Create: `crates/kali_blast_radius/src/register.rs`
- Test: `crates/kali_blast_radius/src/register_tests.rs`
- Modify: `Cargo.toml` (workspace members + workspace.dependencies)

**Interfaces:**
- Consumes: nothing.
- Produces: `RegisterEntry { id: String, tier: u8, title: String }` and
  `parse_register(markdown: &str) -> Result<Vec<RegisterEntry>, String>`.

Background: the register's §2 is organised under four headers — `## Tier 1 — silently drops code or output`, `## Tier 2 — silently produces a wrong value`, `## Tier 3 — silently wrong control flow (value otherwise intact)`, `## Tier 4 — rendering-only (the in-memory value is correct)`. Entries beneath them are `### R-NN: <title>`. Tier comes from the most recent tier header above an entry.

- [ ] **Step 1: Create the crate manifest**

```toml
# crates/kali_blast_radius/Cargo.toml
[package]
name = "kali_blast_radius"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
sha2 = { workspace = true }
```

- [ ] **Step 2: Register the crate in the workspace**

In `/workspace/Cargo.toml`, add `"crates/kali_blast_radius",` to `[workspace] members` immediately after `"crates/kali_case_runner",`, and add to `[workspace.dependencies]` immediately after the `kali_case_runner` line:

```toml
kali_blast_radius = { path = "crates/kali_blast_radius" }
```

- [ ] **Step 3: Write the failing test**

```rust
// crates/kali_blast_radius/src/register_tests.rs
use super::*;

const SAMPLE: &str = "\
## Tier 1 — silently drops code or output

### R-01: A default parameter silently truncates the rest of the module

prose

### R-49: `parse_switch_statement` reparented every post-switch statement — **CLOSED 2026-07-28**

prose

## Tier 2 — silently produces a wrong value

### R-13: Computed member access with a variable key — reads return `0`

prose

## Tier 4 — rendering-only (the in-memory value is correct)

### R-33: `console.warn` `[warn]` prefix

prose
";

#[test]
fn parses_id_tier_and_title_for_every_entry() {
    let entries = parse_register(SAMPLE).expect("sample parses");
    assert_eq!(entries.len(), 4);
    assert_eq!(entries[0].id, "R-01");
    assert_eq!(entries[0].tier, 1);
    assert_eq!(
        entries[0].title,
        "A default parameter silently truncates the rest of the module"
    );
    assert_eq!(entries[1].id, "R-49");
    assert_eq!(entries[1].tier, 1);
    assert_eq!(entries[2].id, "R-13");
    assert_eq!(entries[2].tier, 2);
    assert_eq!(entries[3].id, "R-33");
    assert_eq!(entries[3].tier, 4);
}

#[test]
fn an_entry_before_any_tier_header_is_an_error() {
    let orphan = "### R-99: stray entry with no tier\n";
    let error = parse_register(orphan).expect_err("an untiered entry must not be silently tiered");
    assert!(error.contains("R-99"), "error names the entry: {error}");
}

#[test]
fn a_duplicate_entry_id_is_an_error() {
    let dupe = "## Tier 1 — x\n\n### R-01: first\n\n### R-01: second\n";
    let error = parse_register(dupe).expect_err("a duplicate id must not be silently merged");
    assert!(error.contains("R-01"), "error names the id: {error}");
}

#[test]
fn parses_the_real_register_and_finds_all_four_tiers() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/superpowers/followups/kali-silent-miscompile-register.md"
    ))
    .expect("the register is readable");
    let entries = parse_register(&text).expect("the real register parses");
    assert!(
        entries.len() >= 40,
        "expected the register's ~42 entries, got {}",
        entries.len()
    );
    for tier in 1..=4u8 {
        assert!(
            entries.iter().any(|entry| entry.tier == tier),
            "no entry parsed at tier {tier} -- the tier headers moved"
        );
    }
}
```

- [ ] **Step 4: Run the test to verify it fails**

Run: `cargo test -p kali_blast_radius`
Expected: FAIL — the crate does not compile (`parse_register` not found).

- [ ] **Step 5: Write the implementation**

```rust
// crates/kali_blast_radius/src/register.rs
//! Parse the silent-miscompile register's §2 into `(id, tier, title)`.
//!
//! Tier is NOT redefined here. It is read off the register's own section
//! headers, which are its existing operational definition of damage kind.

/// One `### R-NN: ...` entry under a `## Tier N` header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterEntry {
    pub id: String,
    pub tier: u8,
    pub title: String,
}

/// The tier a `## Tier N — ...` header declares, or `None` for any other
/// heading. Matched on the `Tier ` prefix rather than the full header text
/// because the em-dashed descriptions are prose and will be reworded.
fn tier_of_header(line: &str) -> Option<u8> {
    let rest = line.strip_prefix("## Tier ")?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

/// `### R-13: Computed member access ...` -> `("R-13", "Computed member ...")`.
///
/// The title is cut at the first `:` only. Entry titles routinely contain
/// further colons and markdown, and preserving them verbatim keeps the parsed
/// title diffable against the register's own text.
fn entry_of_header(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix("### R-")?;
    let (number, title) = rest.split_once(':')?;
    if number.is_empty() || !number.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some((format!("R-{number}"), title.trim().to_string()))
}

pub fn parse_register(markdown: &str) -> Result<Vec<RegisterEntry>, String> {
    let mut entries: Vec<RegisterEntry> = Vec::new();
    let mut tier: Option<u8> = None;

    for line in markdown.lines() {
        if let Some(found) = tier_of_header(line) {
            tier = Some(found);
            continue;
        }
        let Some((id, title)) = entry_of_header(line) else {
            continue;
        };
        let Some(tier) = tier else {
            return Err(format!(
                "entry `{id}` appears before any `## Tier N` header -- it has no tier, and \
                 guessing one would invent the axis this measurement exists to avoid inventing"
            ));
        };
        if let Some(previous) = entries.iter().find(|entry| entry.id == id) {
            return Err(format!(
                "entry `{id}` appears twice (tier {} then tier {tier}) -- two records for one id \
                 would be counted twice in the ranking",
                previous.tier
            ));
        }
        entries.push(RegisterEntry { id, tier, title });
    }

    if entries.is_empty() {
        return Err("no `### R-NN:` entries found -- the register's §2 shape changed".to_string());
    }
    Ok(entries)
}

#[cfg(test)]
#[path = "register_tests.rs"]
mod register_tests;
```

```rust
// crates/kali_blast_radius/src/lib.rs
//! Pure logic for the blast-radius measurement: register parsing, the
//! predicate catalogue, verdict classification, and Pareto banding.
//!
//! Deliberately a leaf crate with no kali dependencies. Everything here is
//! unit-testable without running a compiler or a process, which is what lets
//! the instruments be validated before they are trusted -- see
//! `docs/superpowers/specs/2026-08-15-blast-radius-ranking-design.md` §10.

mod register;
pub use register::{parse_register, RegisterEntry};
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p kali_blast_radius`
Expected: PASS, 4 tests.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml crates/kali_blast_radius
git commit -m "feat(blast-radius): read entry ids and tiers off the register itself"
```

---

## Task 2: Predicate catalogue and its completeness gate

**Files:**
- Create: `tools/blast-radius/predicates.json`
- Create: `crates/kali_blast_radius/src/catalogue.rs`
- Test: `crates/kali_blast_radius/src/catalogue_tests.rs`
- Modify: `crates/kali_blast_radius/src/lib.rs`

**Interfaces:**
- Consumes: `RegisterEntry`, `parse_register` (Task 1).
- Produces: `Predicate`, `CatalogueEntry { id, predicate }`,
  `parse_catalogue(json: &str) -> Result<Vec<CatalogueEntry>, String>`,
  `check_completeness(&[RegisterEntry], &[CatalogueEntry]) -> Result<(), String>`.
  The `matcher` string on a countable predicate is the exported function name
  in `tools/blast-radius/matchers.mjs` (Task 12).

This is the task that fires the spec's headline risk (§12) early: if most entries turn out uncountable, the frequency axis is too thin to rank on and the project should stop and re-plan rather than fall through to a tier-only ranking.

- [ ] **Step 1: Write the failing test**

```rust
// crates/kali_blast_radius/src/catalogue_tests.rs
use super::*;

const SAMPLE: &str = r#"{
  "entries": [
    { "id": "R-13", "kind": "countable",
      "matcher": "computedMemberNonLiteralKey",
      "description": "computed member access whose key expression is not a literal" },
    { "id": "R-16", "kind": "uncountable",
      "reason": "a representation condition (handle leaks in concat position), not a syntactic shape" }
  ]
}"#;

#[test]
fn parses_both_predicate_kinds() {
    let entries = parse_catalogue(SAMPLE).expect("sample parses");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].id, "R-13");
    match &entries[0].predicate {
        Predicate::Countable { matcher, .. } => assert_eq!(matcher, "computedMemberNonLiteralKey"),
        Predicate::Uncountable { .. } => panic!("R-13 is countable"),
    }
    match &entries[1].predicate {
        Predicate::Uncountable { reason } => assert!(!reason.is_empty()),
        Predicate::Countable { .. } => panic!("R-16 is uncountable"),
    }
}

#[test]
fn an_uncountable_entry_with_an_empty_reason_is_rejected() {
    let json = r#"{"entries":[{"id":"R-16","kind":"uncountable","reason":"  "}]}"#;
    let error = parse_catalogue(json).expect_err("an unexplained uncountable must be rejected");
    assert!(error.contains("R-16"), "error names the entry: {error}");
}

#[test]
fn completeness_rejects_a_register_entry_with_no_catalogue_record() {
    let register = vec![
        RegisterEntry { id: "R-13".into(), tier: 2, title: "t".into() },
        RegisterEntry { id: "R-99".into(), tier: 2, title: "t".into() },
    ];
    let catalogue = parse_catalogue(SAMPLE).expect("sample parses");
    let error = check_completeness(&register, &catalogue)
        .expect_err("a missing record must fail, not default to uncountable");
    assert!(error.contains("R-99"), "error names the gap: {error}");
}

#[test]
fn completeness_rejects_a_catalogue_record_for_an_unknown_entry() {
    let register = vec![RegisterEntry { id: "R-13".into(), tier: 2, title: "t".into() }];
    let catalogue = parse_catalogue(SAMPLE).expect("sample parses");
    let error = check_completeness(&register, &catalogue)
        .expect_err("a stale record must fail so the catalogue cannot drift");
    assert!(error.contains("R-16"), "error names the stale record: {error}");
}

#[test]
fn the_shipped_catalogue_covers_the_real_register_exactly() {
    let register_text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/superpowers/followups/kali-silent-miscompile-register.md"
    ))
    .expect("the register is readable");
    let catalogue_text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/blast-radius/predicates.json"
    ))
    .expect("the catalogue is readable");
    let register = parse_register(&register_text).expect("register parses");
    let catalogue = parse_catalogue(&catalogue_text).expect("catalogue parses");
    check_completeness(&register, &catalogue).expect("catalogue covers the register exactly");
    assert!(
        !catalogue.is_empty(),
        "an empty catalogue must not report completeness -- that is a ran-nothing-green"
    );
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p kali_blast_radius catalogue`
Expected: FAIL — `parse_catalogue` not found.

- [ ] **Step 3: Write the implementation**

```rust
// crates/kali_blast_radius/src/catalogue.rs
//! The predicate catalogue: what each register entry's triggering construct
//! is, or why it has none.
//!
//! An entry with no syntactic predicate is `uncountable` WITH A WRITTEN
//! REASON. That is not bookkeeping: §0.1 of the register declined to rank the
//! frontier precisely because no frequency model existed, and filling the gap
//! with an estimate would repeat the failure it refused. Uncountable entries
//! band on tier alone (see `score.rs`).

use crate::register::RegisterEntry;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Predicate {
    /// `matcher` is the exported function name in `tools/blast-radius/matchers.mjs`.
    Countable { matcher: String, description: String },
    Uncountable { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogueEntry {
    pub id: String,
    pub predicate: Predicate,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalogue {
    entries: Vec<RawEntry>,
}

// `deny_unknown_fields` is load-bearing here for the same reason it is in the
// case runner's model: a typo'd key must fail the load, not silently produce a
// record that measures nothing.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
struct RawEntry {
    id: String,
    kind: RawKind,
    #[serde(default)]
    matcher: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum RawKind {
    Countable,
    Uncountable,
}

pub fn parse_catalogue(json: &str) -> Result<Vec<CatalogueEntry>, String> {
    let raw: RawCatalogue =
        serde_json::from_str(json).map_err(|error| format!("catalogue is not valid json: {error}"))?;

    let mut out = Vec::with_capacity(raw.entries.len());
    for entry in raw.entries {
        let id = entry.id;
        let predicate = match entry.kind {
            RawKind::Countable => {
                let matcher = non_blank(entry.matcher, &id, "matcher")?;
                let description = non_blank(entry.description, &id, "description")?;
                if entry.reason.is_some() {
                    return Err(format!("`{id}` is countable but also sets `reason`"));
                }
                Predicate::Countable { matcher, description }
            }
            RawKind::Uncountable => {
                let reason = non_blank(entry.reason, &id, "reason")?;
                if entry.matcher.is_some() || entry.description.is_some() {
                    return Err(format!(
                        "`{id}` is uncountable but also sets `matcher`/`description`"
                    ));
                }
                Predicate::Uncountable { reason }
            }
        };
        out.push(CatalogueEntry { id, predicate });
    }
    Ok(out)
}

/// A field that is absent, empty, or all whitespace is rejected rather than
/// accepted as a value. An `uncountable` with a blank reason is exactly the
/// unexplained exclusion this catalogue exists to prevent.
fn non_blank(value: Option<String>, id: &str, field: &str) -> Result<String, String> {
    match value {
        Some(text) if !text.trim().is_empty() => Ok(text),
        _ => Err(format!("`{id}` has no `{field}` -- it must be written down, not left blank")),
    }
}

/// Every register entry has exactly one catalogue record, and every catalogue
/// record names a real register entry. Both directions matter: the first stops
/// an entry vanishing by omission, the second stops the catalogue keeping
/// records for entries that were renamed or removed.
pub fn check_completeness(
    register: &[RegisterEntry],
    catalogue: &[CatalogueEntry],
) -> Result<(), String> {
    let mut missing: Vec<&str> = Vec::new();
    for entry in register {
        let hits = catalogue.iter().filter(|record| record.id == entry.id).count();
        if hits == 0 {
            missing.push(&entry.id);
        } else if hits > 1 {
            return Err(format!("`{}` has {hits} catalogue records -- expected exactly 1", entry.id));
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "register entries with no catalogue record: {} -- every entry needs a predicate or an \
             explicit `uncountable` reason",
            missing.join(", ")
        ));
    }

    let stale: Vec<&str> = catalogue
        .iter()
        .filter(|record| !register.iter().any(|entry| entry.id == record.id))
        .map(|record| record.id.as_str())
        .collect();
    if !stale.is_empty() {
        return Err(format!(
            "catalogue records naming no register entry: {} -- the catalogue has drifted",
            stale.join(", ")
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "catalogue_tests.rs"]
mod catalogue_tests;
```

Add to `crates/kali_blast_radius/src/lib.rs`:

```rust
mod catalogue;
pub use catalogue::{check_completeness, parse_catalogue, CatalogueEntry, Predicate};
```

- [ ] **Step 4: Author the catalogue**

Create `tools/blast-radius/predicates.json` with **one record per `### R-NN:` entry in the register's §2** — read the register and enumerate them; do not guess the list. For each entry, read its §2 body and decide:

- **countable** — the entry's repro reduces to a shape acorn can match. Write a `matcher` name in `lowerCamelCase` and a one-line `description` stating the shape precisely.
- **uncountable** — it does not. Write a `reason` saying what kind of condition it is instead.

Worked examples, verbatim-usable:

```json
{
  "entries": [
    { "id": "R-10", "kind": "countable",
      "matcher": "shadowingBlockDeclaration",
      "description": "a let/const declaration in a nested block whose declared name is also bound in an enclosing scope" },
    { "id": "R-13", "kind": "countable",
      "matcher": "computedMemberNonLiteralKey",
      "description": "computed member access whose key expression is not a literal" },
    { "id": "R-14", "kind": "countable",
      "matcher": "memberReadOnCallResult",
      "description": "a member or computed read applied directly to a call expression's result" },
    { "id": "R-16", "kind": "uncountable",
      "reason": "a representation condition (a string handle leaking in concat position), not a syntactic shape acorn can match" }
  ]
}
```

Every entry recorded as `CLOSED` in the register still gets a record — closed entries are re-measured too (Task 7-9), and a record is how the completeness gate proves none was skipped. A closed entry is normally `countable` if its shape is syntactic; mark it `uncountable` only for the same reasons any other entry would be.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p kali_blast_radius`
Expected: PASS. If `the_shipped_catalogue_covers_the_real_register_exactly` fails, the catalogue is missing entries — add them; do not relax the test.

- [ ] **Step 6: Check the headline risk before continuing**

Run:

```bash
python3 -c "
import json
e = json.load(open('tools/blast-radius/predicates.json'))['entries']
u = [x['id'] for x in e if x['kind'] == 'uncountable']
print(f'{len(e)} entries, {len(u)} uncountable: {u}')
"
```

If **more than half** are uncountable, **STOP**. The frequency axis is too thin to band on, and pushing through produces the tier-only ranking spec §3.3 rejects. Report the count and the uncountable list, and ask for a re-plan before starting Task 3.

- [ ] **Step 7: Commit**

```bash
git add tools/blast-radius/predicates.json crates/kali_blast_radius
git commit -m "feat(blast-radius): the predicate catalogue, with uncountable stated not estimated"
```

---

## Task 3: Verdict classifier

**Files:**
- Create: `crates/kali_blast_radius/src/verdict.rs`
- Test: `crates/kali_blast_radius/src/verdict_tests.rs`
- Modify: `crates/kali_blast_radius/src/lib.rs`

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces: `Run { code: Option<i32>, stdout: String, stderr: String, timed_out: bool }`,
  `Verdict` (8 variants), `classify(kali: &Run, node: &Run) -> Verdict`,
  `runs_agree(a: &Run, b: &Run) -> bool`, `is_documented_code(code: &str) -> bool`,
  `Verdict::as_str(self) -> &'static str`.

The classification table is spec §7. `is_documented_code` follows `specs/15-errors.md`'s public range registry: `E51xx`, `E52xx`, `E53xx`, `E54xx`, `E55xx`, `E6xxx`, `E7xxx`, `E8xxx`, `E9xxx` are documented; `E0xxx` is internal; anything else — including the real-but-undocumented `E4003`/`E4201` — is not documented.

- [ ] **Step 1: Write the failing test**

```rust
// crates/kali_blast_radius/src/verdict_tests.rs
use super::*;

fn ok(stdout: &str) -> Run {
    Run { code: Some(0), stdout: stdout.into(), stderr: String::new(), timed_out: false }
}

fn failed(code: i32, stderr: &str) -> Run {
    Run { code: Some(code), stdout: String::new(), stderr: stderr.into(), timed_out: false }
}

fn timed_out() -> Run {
    Run { code: None, stdout: String::new(), stderr: String::new(), timed_out: true }
}

#[test]
fn equal_output_on_both_sides_is_fixed() {
    assert_eq!(classify(&ok("7\n"), &ok("7\n")), Verdict::Fixed);
}

#[test]
fn exit_zero_both_sides_with_different_output_is_silent() {
    assert_eq!(classify(&ok("0\n"), &ok("1\n")), Verdict::Silent);
}

#[test]
fn a_documented_denial_against_a_working_node_is_fail_closed() {
    let kali = failed(1, "error[E5506]: feature unavailable in current phase");
    assert_eq!(classify(&kali, &ok("1\n")), Verdict::FailClosed);
}

#[test]
fn an_internal_e0xxx_against_a_working_node_is_fl_internal() {
    let kali = failed(1, "error[E0001]: internal compiler error");
    assert_eq!(classify(&kali, &ok("1\n")), Verdict::FlInternal);
}

#[test]
fn the_undocumented_e4xxx_family_is_fl_internal() {
    // E4003 (fuel trap) and E4201 (wasm translation) are real and reachable but
    // absent from specs/15-errors.md's range table -- see spec §7.1.
    assert_eq!(classify(&failed(1, "error[E4003]: trap"), &ok("x\n")), Verdict::FlInternal);
    assert_eq!(classify(&failed(1, "error[E4201]: translation"), &ok("x\n")), Verdict::FlInternal);
}

#[test]
fn kali_accepting_what_node_refuses_is_accepts_invalid() {
    let node = failed(1, "SyntaxError: More than one default clause in switch statement");
    assert_eq!(classify(&ok("v=d2\n"), &node), Verdict::AcceptsInvalid);
}

#[test]
fn both_refusing_is_both_reject() {
    let kali = failed(1, "error[E5506]: nope");
    let node = failed(1, "SyntaxError: nope");
    assert_eq!(classify(&kali, &node), Verdict::BothReject);
}

#[test]
fn a_timeout_on_either_side_outranks_every_other_verdict() {
    assert_eq!(classify(&timed_out(), &ok("1\n")), Verdict::Timeout);
    assert_eq!(classify(&ok("1\n"), &timed_out()), Verdict::Timeout);
}

#[test]
fn a_denial_with_no_recognisable_code_is_fl_internal_not_fail_closed() {
    // A panic or an unadorned failure is not an honest denial. Defaulting it to
    // FAIL_CLOSED would count a crash as acceptable behaviour.
    let kali = failed(101, "thread 'main' panicked at src/main.rs:1:1");
    assert_eq!(classify(&kali, &ok("1\n")), Verdict::FlInternal);
}

#[test]
fn runs_agree_compares_output_and_exit_but_two_timeouts_never_agree() {
    assert!(runs_agree(&ok("a"), &ok("a")));
    assert!(!runs_agree(&ok("a"), &ok("b")));
    assert!(!runs_agree(&failed(1, "x"), &failed(2, "x")));
    // A pair of timeouts is not evidence of stable behaviour.
    assert!(!runs_agree(&timed_out(), &timed_out()));
}

#[test]
fn documented_ranges_follow_the_errors_spec() {
    for code in ["E5101", "E5203", "E5506", "E6004", "E7001", "E8002", "E9100"] {
        assert!(is_documented_code(code), "{code} is in the spec's public ranges");
    }
    for code in ["E0001", "E4003", "E4201", "E1000", "W3002", "nonsense"] {
        assert!(!is_documented_code(code), "{code} is not a documented error code");
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p kali_blast_radius verdict`
Expected: FAIL — `classify` not found.

- [ ] **Step 3: Write the implementation**

```rust
// crates/kali_blast_radius/src/verdict.rs
//! Derive a verdict class from one kali run and one node run.
//!
//! The class -- not the literal output -- is what an oracle case asserts. That
//! is the whole point: the register's §0.2 has been stale since 2026-07-24
//! because a verdict was prose a human had to re-derive. As a class, a change
//! is a red test.

/// One side's captured process result. `code` is `None` when the process was
/// killed (timeout) or died to a signal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Fixed,
    Silent,
    FailClosed,
    FlInternal,
    AcceptsInvalid,
    BothReject,
    Timeout,
    Nondeterministic,
}

impl Verdict {
    /// The spelling a case file writes in `verdict = "..."`.
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Fixed => "fixed",
            Verdict::Silent => "silent",
            Verdict::FailClosed => "fail_closed",
            Verdict::FlInternal => "fl_internal",
            Verdict::AcceptsInvalid => "accepts_invalid",
            Verdict::BothReject => "both_reject",
            Verdict::Timeout => "timeout",
            Verdict::Nondeterministic => "nondeterministic",
        }
    }
}

/// Is `code` in `specs/15-errors.md`'s public range registry, excluding the
/// `E0xxx` internal family?
///
/// `E4xxx` is deliberately NOT documented-in-spec: `E4003` (fuel trap) and
/// `E4201` (WebAssembly translation error) are real and reachable, but the
/// spec's range table has no `E4xxx` row at all. They therefore classify as
/// `FL_INTERNAL` -- the right verdict, currently for the wrong reason. Task 4
/// closes the taxonomy gap; this function is not where an exception list goes,
/// because hiding a spec gap inside a test tool is how it stays open.
pub fn is_documented_code(code: &str) -> bool {
    let Some(digits) = code.strip_prefix('E') else {
        return false;
    };
    if digits.len() != 4 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    matches!(digits.as_bytes()[0], b'5' | b'6' | b'7' | b'8' | b'9')
}

/// The first `error[Ennnn]` code in a captured stderr, if any.
fn first_error_code(stderr: &str) -> Option<String> {
    let start = stderr.find("error[")? + "error[".len();
    let rest = &stderr[start..];
    let end = rest.find(']')?;
    Some(rest[..end].to_string())
}

fn refused(run: &Run) -> bool {
    run.code != Some(0)
}

/// Two runs of the same side, compared. Used to detect nondeterminism before a
/// verdict is recorded.
///
/// Two timeouts do NOT agree. A pair of hangs says the program never settled,
/// which is not evidence that its behaviour is stable.
pub fn runs_agree(a: &Run, b: &Run) -> bool {
    if a.timed_out || b.timed_out {
        return false;
    }
    a.code == b.code && a.stdout == b.stdout && a.stderr == b.stderr
}

pub fn classify(kali: &Run, node: &Run) -> Verdict {
    if kali.timed_out || node.timed_out {
        return Verdict::Timeout;
    }
    match (refused(kali), refused(node)) {
        (false, false) => {
            if kali.stdout == node.stdout {
                Verdict::Fixed
            } else {
                Verdict::Silent
            }
        }
        (true, false) => match first_error_code(&kali.stderr) {
            Some(code) if is_documented_code(&code) => Verdict::FailClosed,
            // No code at all (a panic, a bare nonzero exit) is not an honest
            // denial either -- defaulting it to FAIL_CLOSED would record a
            // crash as acceptable.
            _ => Verdict::FlInternal,
        },
        (false, true) => Verdict::AcceptsInvalid,
        (true, true) => Verdict::BothReject,
    }
}

#[cfg(test)]
#[path = "verdict_tests.rs"]
mod verdict_tests;
```

Add to `crates/kali_blast_radius/src/lib.rs`:

```rust
mod verdict;
pub use verdict::{classify, is_documented_code, runs_agree, Run, Verdict};
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p kali_blast_radius`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_blast_radius
git commit -m "feat(blast-radius): a verdict class an oracle case can assert"
```

---

## Task 4: Document the `E4xxx` family

**Files:**
- Modify: `specs/15-errors.md` (the "Expanded public ranges used in schema v1" table, currently between the `E0xxx` and `E51xx` rows)

**Interfaces:**
- Consumes: nothing.
- Produces: nothing in code. Closes the taxonomy gap `verdict.rs` documents.

`E4003` (fuel trap) and `E4201` (WebAssembly translation error) exist in real behaviour and are what the register means by `FL-INTERNAL`, but the spec's public range table has no `E4xxx` row. The classifier currently reaches the right verdict for the wrong reason.

- [ ] **Step 1: Confirm the codes and their meanings**

```bash
grep -rn "E4003\|E4201" crates/ --include=*.rs | head -20
```

Read each hit and write down what actually emits each code. Do not describe a code you have not traced to its emitter.

- [ ] **Step 2: Add the range row**

In `specs/15-errors.md`, add a row to the expanded-range table immediately after the `| E0xxx | Internal compiler errors |` row:

```markdown
| E4xxx | Wasm translation and execution-engine failures (internal — not an availability denial) |
```

- [ ] **Step 3: Add the clarifying prose**

In the "Range clarification" bullet list immediately below that table, add:

```markdown
- `E4xxx` is the wasm translation/execution-engine family (for example `E4201`
  WebAssembly translation failure, `E4003` a fuel trap). It is **internal**, in
  the same sense as `E0xxx`: it reports that kali failed, not that the user
  asked for something unavailable. It must never be read as an honest
  availability denial — that is `E5506`'s job — and tooling that separates
  honest denials from internal failures must classify `E4xxx` with `E0xxx`.
```

- [ ] **Step 4: Verify the classifier still agrees**

Run: `cargo test -p kali_blast_radius verdict`
Expected: PASS — `the_undocumented_e4xxx_family_is_fl_internal` still passes, because `is_documented_code` keys on the numeric range and `E4xxx` is documented *as internal*, not as an honest denial. If it fails, the implementation drifted from the spec; fix the implementation, not the test.

- [ ] **Step 5: Rename the stale test**

The test name now overstates. Rename it in `crates/kali_blast_radius/src/verdict_tests.rs`:

```rust
#[test]
fn the_internal_e4xxx_family_is_fl_internal() {
```

and update its comment to:

```rust
    // E4003 (fuel trap) and E4201 (wasm translation) are the internal E4xxx
    // family -- documented as internal in specs/15-errors.md, never an honest
    // denial. See spec §7.1.
```

Also update the doc comment on `is_documented_code` in `crates/kali_blast_radius/src/verdict.rs`, replacing the paragraph beginning "`E4xxx` is deliberately NOT documented-in-spec" with:

```rust
/// `E4xxx` is documented in `specs/15-errors.md` as an INTERNAL family (wasm
/// translation and execution-engine failures), alongside `E0xxx`. Neither is
/// an honest availability denial, so neither counts as documented here.
```

- [ ] **Step 6: Run the tests**

Run: `cargo test -p kali_blast_radius`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add specs/15-errors.md crates/kali_blast_radius
git commit -m "docs(errors): the E4xxx family exists -- give it a row and call it internal"
```

---

## Task 5: The `oracle` step kind

**Files:**
- Modify: `crates/kali_case_runner/Cargo.toml`
- Modify: `crates/kali_case_runner/src/model.rs`
- Modify: `crates/kali_case_runner/src/model_tests.rs`
- Modify: `crates/kali_case_runner/src/steps.rs`
- Modify: `crates/kali_case_runner/src/steps_tests.rs`

**Interfaces:**
- Consumes: `classify`, `runs_agree`, `Run`, `Verdict` (Task 3).
- Produces: `StepKind::Oracle`; `Step` fields `register_entry: Option<String>`,
  `program: Option<String>`, `verdict: Option<Verdict>`, `timeout_ms: Option<u64>`.
  Case-file syntax:

```toml
[[case]]
name = "..."
kind = "oracle"
register_entry = "R-13"
program = "r13_module.js"
verdict = "silent"
```

Field names avoid `entry` and `body`, which are already `browser_bundle_harness`-only.

- [ ] **Step 1: Add the dependency**

In `crates/kali_case_runner/Cargo.toml`, add to `[dependencies]`:

```toml
kali_blast_radius = { workspace = true }
```

- [ ] **Step 2: Write the failing model tests**

Append to `crates/kali_case_runner/src/model_tests.rs`:

```rust
#[test]
fn an_oracle_step_parses_its_four_fields() {
    let text = r#"
[[case]]
name = "c"
kind = "oracle"
register_entry = "R-13"
program = "r13.js"
verdict = "silent"
timeout_ms = 5000
"#;
    let file = parse_case_file(text).expect("parses");
    let step = file.case[0].inline.as_ref().expect("inline step");
    assert_eq!(step.kind, StepKind::Oracle);
    assert_eq!(step.register_entry.as_deref(), Some("R-13"));
    assert_eq!(step.program.as_deref(), Some("r13.js"));
    assert_eq!(step.verdict, Some(kali_blast_radius::Verdict::Silent));
    assert_eq!(step.timeout_ms, Some(5000));
}

#[test]
fn oracle_fields_without_an_explicit_kind_are_rejected() {
    // Same rule browser_bundle_harness follows: a forgotten `kind` must not
    // silently become a `cli` step that ignores the fields entirely.
    let text = r#"
[[case]]
name = "c"
program = "r13.js"
verdict = "silent"
"#;
    let error = parse_case_file(text).expect_err("must demand an explicit kind");
    assert!(error.contains("oracle"), "error names the kind: {error}");
}

#[test]
fn an_oracle_step_declaring_stdout_assertions_is_rejected() {
    // An oracle step asserts a derived class. A `stdout` claim on it would
    // never be evaluated -- parses clean, asserts nothing.
    let text = r#"
[[case]]
name = "c"
kind = "oracle"
register_entry = "R-13"
program = "r13.js"
verdict = "silent"
stdout = "1\n"
"#;
    let error = parse_case_file(text).expect_err("must reject inapplicable assertions");
    assert!(error.contains("stdout"), "error names the field: {error}");
}

#[test]
fn an_oracle_step_missing_verdict_is_rejected() {
    let text = r#"
[[case]]
name = "c"
kind = "oracle"
register_entry = "R-13"
program = "r13.js"
"#;
    let error = parse_case_file(text).expect_err("a case with no verdict asserts nothing");
    assert!(error.contains("verdict"), "error names the field: {error}");
}

#[test]
fn an_oracle_step_missing_register_entry_is_rejected() {
    let text = r#"
[[case]]
name = "c"
kind = "oracle"
program = "r13.js"
verdict = "silent"
"#;
    let error = parse_case_file(text).expect_err("an unattributed verdict cannot regenerate §0.2");
    assert!(error.contains("register_entry"), "error names the field: {error}");
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p kali_case_runner model`
Expected: FAIL — `StepKind::Oracle` does not exist.

- [ ] **Step 4: Implement the model changes**

In `crates/kali_case_runner/src/model.rs`:

Add the variant to `StepKind` and its spelling:

```rust
pub enum StepKind {
    #[default]
    Cli,
    FileJson,
    BrowserBundleHarness,
    Oracle,
}
```

```rust
            StepKind::BrowserBundleHarness => "browser_bundle_harness",
            StepKind::Oracle => "oracle",
```

Add a deserializable wrapper for the verdict word, next to `ExitStatusWord`:

```rust
/// `verdict = "silent"` in a case file. A thin serde front for
/// `kali_blast_radius::Verdict`, which is a plain enum in a leaf crate with no
/// serde dependency of its own.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum RawVerdict {
    Fixed,
    Silent,
    FailClosed,
    FlInternal,
    AcceptsInvalid,
    BothReject,
    Timeout,
    Nondeterministic,
}

impl From<RawVerdict> for kali_blast_radius::Verdict {
    fn from(raw: RawVerdict) -> Self {
        use kali_blast_radius::Verdict as V;
        match raw {
            RawVerdict::Fixed => V::Fixed,
            RawVerdict::Silent => V::Silent,
            RawVerdict::FailClosed => V::FailClosed,
            RawVerdict::FlInternal => V::FlInternal,
            RawVerdict::AcceptsInvalid => V::AcceptsInvalid,
            RawVerdict::BothReject => V::BothReject,
            RawVerdict::Timeout => V::Timeout,
            RawVerdict::Nondeterministic => V::Nondeterministic,
        }
    }
}
```

Add to `RawStep`, after the `body` field:

```rust
    #[serde(default)]
    register_entry: Option<String>,
    #[serde(default)]
    program: Option<String>,
    #[serde(default)]
    verdict: Option<RawVerdict>,
    #[serde(default)]
    timeout_ms: Option<u64>,
```

Add to `Step`, after the `body` field:

```rust
    /// `oracle` only: the register entry this case re-measures (`"R-13"`).
    /// Required, because an unattributed verdict cannot regenerate §0.2.
    pub register_entry: Option<String>,
    /// `oracle` only: the `[source]` key both engines run.
    pub program: Option<String>,
    /// `oracle` only: the verdict class the run must produce.
    pub verdict: Option<kali_blast_radius::Verdict>,
    /// `oracle` only: per-run wall-clock budget. Defaults to
    /// `steps::ORACLE_DEFAULT_TIMEOUT_MS` when unset.
    pub timeout_ms: Option<u64>,
```

In `finalize_step`, extend kind inference and applicability. Add alongside the existing `wants_file_json` / `wants_browser`:

```rust
    let wants_oracle = raw.register_entry.is_some()
        || raw.program.is_some()
        || raw.verdict.is_some()
        || raw.timeout_ms.is_some();
```

Add a no-explicit-kind arm to the `match raw.kind` block, immediately after the existing `None if wants_browser =>` arm and before `None => StepKind::default()`, so an oracle field without `kind` is an error rather than a silent `cli` step:

```rust
        None if wants_oracle => {
            return Err(format!(
                "case `{case_name}`: step sets `register_entry`, `program`, `verdict` or \
                 `timeout_ms`, which requires an explicit `kind = \"oracle\"` -- `kind` only \
                 defaults to `cli` when no kind-specific field is set"
            ));
        }
```

The existing `wants_file_json && wants_browser` conflict arm must also learn about oracle. Replace it with a count-based check that names whichever kinds collided:

```rust
        None if [wants_file_json, wants_browser, wants_oracle].iter().filter(|w| **w).count() > 1 => {
            return Err(format!(
                "case `{case_name}`: step sets fields belonging to more than one kind \
                 (`path`/`fields` are file_json-only, `entry`/`body` are \
                 browser_bundle_harness-only, `register_entry`/`program`/`verdict`/`timeout_ms` \
                 are oracle-only) without an explicit `kind`"
            ));
        }
```

Then extend the existing `match kind` applicability block — the one that accumulates into `inapplicable: Vec<&'static str>` — with an `Oracle` arm. Add the required-field checks first, since a missing one is a different error from an inapplicable one:

```rust
        StepKind::Oracle => {
            if raw.register_entry.is_none() {
                return Err(format!(
                    "case `{case_name}`: an `oracle` step requires `register_entry` -- a verdict \
                     that names no entry cannot regenerate the register's §0.2"
                ));
            }
            if raw.program.is_none() {
                return Err(format!(
                    "case `{case_name}`: an `oracle` step requires `program` -- the `[source]` \
                     key both engines run"
                ));
            }
            if raw.verdict.is_none() {
                return Err(format!(
                    "case `{case_name}`: an `oracle` step requires `verdict` -- without it the \
                     step runs both engines and asserts nothing"
                ));
            }
            // An oracle step asserts a derived class, never raw output. A
            // process-output assertion on it would parse clean and never be
            // evaluated -- the exact degradation this format exists to close.
            if !raw.args.is_empty() {
                inapplicable.push("args");
            }
            if raw.exit.is_some() {
                inapplicable.push("exit");
            }
            if raw.stdout.is_some() {
                inapplicable.push("stdout");
            }
            if !raw.stdout_contains.is_empty() {
                inapplicable.push("stdout_contains");
            }
            if !raw.stdout_absent.is_empty() {
                inapplicable.push("stdout_absent");
            }
            if !raw.stdout_count.is_empty() {
                inapplicable.push("stdout_count");
            }
            if raw.stderr.is_some() {
                inapplicable.push("stderr");
            }
            if !raw.stderr_contains.is_empty() {
                inapplicable.push("stderr_contains");
            }
            if !raw.stderr_absent.is_empty() {
                inapplicable.push("stderr_absent");
            }
            if raw.json.is_some() {
                inapplicable.push("json");
            }
            if !raw.json_null.is_empty() {
                inapplicable.push("json_null");
            }
            if !raw.json_count.is_empty() {
                inapplicable.push("json_count");
            }
            if raw.path.is_some() {
                inapplicable.push("path");
            }
            if raw.fields.is_some() {
                inapplicable.push("fields");
            }
            if raw.entry.is_some() {
                inapplicable.push("entry");
            }
            if raw.body.is_some() {
                inapplicable.push("body");
            }
        }
```

The three non-oracle arms of that same `match` must each reject the four oracle-only fields, exactly as they already reject each other's:

```rust
            if raw.register_entry.is_some() {
                inapplicable.push("register_entry");
            }
            if raw.program.is_some() {
                inapplicable.push("program");
            }
            if raw.verdict.is_some() {
                inapplicable.push("verdict");
            }
            if raw.timeout_ms.is_some() {
                inapplicable.push("timeout_ms");
            }
```

Carry the four new fields through into the constructed `Step`:

```rust
        register_entry: raw.register_entry,
        program: raw.program,
        verdict: raw.verdict.map(Into::into),
        timeout_ms: raw.timeout_ms,
```

Every other kind's `Step` construction must set these to `None`.

- [ ] **Step 5: Run the model tests to verify they pass**

Run: `cargo test -p kali_case_runner model`
Expected: PASS.

- [ ] **Step 6: Commit the model half**

```bash
git add crates/kali_case_runner/Cargo.toml crates/kali_case_runner/src/model.rs crates/kali_case_runner/src/model_tests.rs
git commit -m "feat(case-runner): an oracle step kind that can only assert a verdict"
```

- [ ] **Step 7: Write the failing execution tests**

Append to `crates/kali_case_runner/src/steps_tests.rs`:

```rust
#[test]
fn a_timeout_is_reported_as_a_timeout_not_a_hang() {
    let mut command = std::process::Command::new("sleep");
    command.arg("30");
    let run = crate::steps::run_with_timeout(command, std::time::Duration::from_millis(100))
        .expect("spawns");
    assert!(run.timed_out, "a killed process must report timed_out");
}

#[test]
fn a_fast_process_is_captured_whole() {
    let mut command = std::process::Command::new("sh");
    command.arg("-c").arg("printf 'out'; printf 'err' 1>&2; exit 3");
    let run = crate::steps::run_with_timeout(command, std::time::Duration::from_secs(10))
        .expect("spawns");
    assert!(!run.timed_out);
    assert_eq!(run.code, Some(3));
    assert_eq!(run.stdout, "out");
    assert_eq!(run.stderr, "err");
}

#[test]
fn a_large_output_does_not_deadlock_on_the_pipe_buffer() {
    // Without concurrent draining, a child writing more than the pipe buffer
    // (64 KiB on Linux) blocks forever and the timeout fires -- turning a
    // working program into a false TIMEOUT verdict.
    let mut command = std::process::Command::new("sh");
    command.arg("-c").arg("yes x | head -c 400000");
    let run = crate::steps::run_with_timeout(command, std::time::Duration::from_secs(30))
        .expect("spawns");
    assert!(!run.timed_out, "large output must not be read as a hang");
    assert_eq!(run.stdout.len(), 400_000);
}
```

- [ ] **Step 8: Run the tests to verify they fail**

Run: `cargo test -p kali_case_runner steps`
Expected: FAIL — `run_with_timeout` not found.

- [ ] **Step 9: Implement timeout-capable execution and `run_oracle`**

In `crates/kali_case_runner/src/steps.rs`, add near the top:

```rust
use kali_blast_radius::{classify, runs_agree, Run, Verdict};
use std::io::Read;
use std::process::Stdio;
use std::time::{Duration, Instant};

/// Per-run wall-clock budget when a case does not set `timeout_ms`.
///
/// Generous on purpose: the cost of a too-short budget is a false `TIMEOUT`
/// verdict recorded against a working program, which corrupts the very table
/// this project exists to make trustworthy. The cost of a long one is a slow
/// failing case.
pub const ORACLE_DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// How often the wait loop wakes to check whether the child has exited.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Run `command` to completion or kill it at `budget`.
///
/// stdout and stderr are drained on their own threads. That is not an
/// optimisation: a child writing past the pipe buffer (64 KiB on Linux) blocks
/// on the write until someone reads, so a single-threaded "wait then read"
/// turns any chatty program into a false `TIMEOUT`. R-09's runaway loops make
/// chatty-and-slow a shape this project measures routinely.
pub fn run_with_timeout(mut command: std::process::Command, budget: Duration) -> Result<Run, String> {
    command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = retry_on_etxtbsy(|| command.spawn())
        .map_err(|error| format!("failed to spawn: {error}"))?;

    let mut stdout_pipe = child.stdout.take().ok_or("no stdout pipe")?;
    let mut stderr_pipe = child.stderr.take().ok_or("no stderr pipe")?;
    let stdout_reader = std::thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = stdout_pipe.read_to_end(&mut buffer);
        buffer
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = stderr_pipe.read_to_end(&mut buffer);
        buffer
    });

    let deadline = Instant::now() + budget;
    let mut timed_out = false;
    let status = loop {
        match child.try_wait().map_err(|error| format!("wait failed: {error}"))? {
            Some(status) => break Some(status),
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                timed_out = true;
                break None;
            }
            None => std::thread::sleep(POLL_INTERVAL),
        }
    };

    let stdout = stdout_reader.join().map_err(|_| "stdout reader panicked".to_string())?;
    let stderr = stderr_reader.join().map_err(|_| "stderr reader panicked".to_string())?;

    Ok(Run {
        code: status.and_then(|status| status.code()),
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
        timed_out,
    })
}

/// The node binary the oracle uses. `KALI_ORACLE_NODE` overrides it so a
/// pinned build can be pointed at without touching any do-not-modify file.
fn oracle_node() -> String {
    std::env::var("KALI_ORACLE_NODE").unwrap_or_else(|_| "node".to_string())
}

fn run_oracle(config: &RunnerConfig, dir: &Path, step: &Step) -> Result<(), String> {
    let program = step.program.as_deref().ok_or("oracle step requires `program`")?;
    let expected = step.verdict.ok_or("oracle step requires `verdict`")?;
    validate_source_key(program)?;
    if !dir.join(program).exists() {
        // A case naming a `[source]` key that does not exist would otherwise
        // run two engines against a missing file, agree that both failed, and
        // pass as BOTH_REJECT having measured nothing.
        return Err(format!(
            "oracle step names `program = \"{program}\"`, which no `[source]` key wrote"
        ));
    }
    let budget = Duration::from_millis(step.timeout_ms.unwrap_or(ORACLE_DEFAULT_TIMEOUT_MS));

    let kali_run = |_: ()| {
        let mut command = Command::new(&config.kali_bin);
        command.current_dir(dir).args(["run", program]);
        for (key, value) in &step.env {
            command.env(key, value);
        }
        run_with_timeout(command, budget)
    };
    let node_run = |_: ()| {
        let mut command = Command::new(oracle_node());
        command.current_dir(dir).arg(program);
        for (key, value) in &step.env {
            command.env(key, value);
        }
        run_with_timeout(command, budget)
    };

    // Both sides run twice. A verdict derived from a single run of a
    // nondeterministic program records whichever answer happened to come out
    // first, which is a measurement that cannot be reproduced -- the failure
    // this whole project is correcting.
    let kali_a = kali_run(())?;
    let kali_b = kali_run(())?;
    let node_a = node_run(())?;
    let node_b = node_run(())?;

    let actual = if kali_a.timed_out || node_a.timed_out {
        Verdict::Timeout
    } else if !runs_agree(&kali_a, &kali_b) || !runs_agree(&node_a, &node_b) {
        Verdict::Nondeterministic
    } else {
        classify(&kali_a, &node_a)
    };

    if actual == expected {
        return Ok(());
    }
    Err(format!(
        "verdict mismatch for {entry}: expected `{}`, measured `{}`\n  \
         kali: exit {:?} stdout {:?} stderr {:?}\n  \
         node: exit {:?} stdout {:?} stderr {:?}",
        expected.as_str(),
        actual.as_str(),
        kali_a.code,
        kali_a.stdout,
        kali_a.stderr,
        node_a.code,
        node_a.stdout,
        node_a.stderr,
        entry = step.register_entry.as_deref().unwrap_or("<no entry>"),
    ))
}
```

Add the dispatch arm in `run_trial`:

```rust
            StepKind::Oracle => run_oracle(config, dir.path(), step),
```

- [ ] **Step 10: Run the tests to verify they pass**

Run: `cargo test -p kali_case_runner`
Expected: PASS.

- [ ] **Step 11: Commit**

```bash
git add crates/kali_case_runner/src/steps.rs crates/kali_case_runner/src/steps_tests.rs
git commit -m "feat(case-runner): run both engines twice, classify, and never call a hang green"
```

---

## Task 6: Ground-truth fixtures for the classifier

**Files:**
- Create: `crates/kali_cli/tests/cases/oracle/classifier_ground_truth.toml`

**Interfaces:**
- Consumes: `StepKind::Oracle` (Task 5).
- Produces: nothing later tasks import. This validates the instrument before it is used for record (spec §10).

Every verdict class gets a case built from a construct whose behaviour is known independently, so a classifier that mislabels a class fails here rather than silently mislabelling 84 entries in Task 7-9.

- [ ] **Step 1: Write the case file**

```toml
# Ground truth for the oracle classifier.
#
# These cases do NOT re-measure register entries -- they check that the
# classifier assigns the class the construct is known to produce. If one of
# these fails, every verdict in `oracle/` is suspect, so this file is the first
# thing to read on a red run.
#
# Spec: docs/superpowers/specs/2026-08-15-blast-radius-ranking-design.md §10.

[source]
"agree.js" = """console.log("7");
"""
"accepts_invalid.js" = """function s(x) {
switch (x) {
default: return "d1";
default: return "d2";
}
}
console.log("v=" + s(1));
"""

[[case]]
name = "matching_output_on_both_engines_classifies_as_fixed"
kind = "oracle"
register_entry = "GROUND-TRUTH"
program = "agree.js"
verdict = "fixed"
rationale = """A bare `console.log` of a string literal is the simplest program both engines agree on. If this does not classify FIXED, the classifier's equality path or the capture path is broken, and no other verdict in this directory can be trusted.

This case is deliberately NOT a register entry: it measures the instrument, not kali."""

[[case]]
name = "a_second_default_clause_classifies_as_accepts_invalid"
kind = "oracle"
register_entry = "R-54"
program = "accepts_invalid.js"
verdict = "accepts_invalid"
rationale = """R-54: `parse_switch_statement`'s `default` arm omits `Default` from its stop set, so a second `default` is absorbed into the first and both bodies run merged. node refuses the whole file with `SyntaxError: More than one default clause in switch statement` (exit 1) while kali exits 0.

This is the only verdict class where node is the side that refuses, so it is the only case that exercises the `(false, true)` arm of `classify`. It doubles as R-54's re-measurement."""
```

- [ ] **Step 2: Run the cases**

Run: `cargo test -p kali_cli --test cases -- oracle/`
Expected: PASS, 2 cases. If `accepts_invalid` reports `both_reject`, R-54 has been fixed since the register recorded it — verify by hand, then update the expected verdict and note the change for Task 10's regeneration.

- [ ] **Step 3: Add the remaining ground-truth classes**

Append cases for `silent`, `fail_closed`, `fl_internal`, `timeout` and `nondeterministic`, using constructs the register documents as producing each. Source each from the register rather than inventing one:

- `silent` — R-13's computed-key repro (`var o = {a:1,b:2}; var k = "a"; console.log("read=" + o[k]);` → kali `read=0`, node `read=1`).
- `fail_closed` — R-20's `JSON.stringify`, which the register records as `E5506`.
- `fl_internal` — R-09's `continue` hang shape, which the register records as reaching `E4003`.
- `timeout` — a program with an unbounded loop that prints nothing, with `timeout_ms = 2000` so the case is quick.
- `nondeterministic` — a program whose output differs run to run (`console.log(String(Date.now()).length > 0 ? Math.random() : 0)` is not usable, because kali may reject it; prefer a program reading `process.hrtime` only if kali accepts it — if no such program is accepted by kali, record that fact in the file's header comment and leave the class covered by `verdict_tests.rs`'s unit test alone rather than fabricating a case).

Each case carries a `rationale` naming the register entry and what class it proves.

- [ ] **Step 4: Run the full oracle directory**

Run: `cargo test -p kali_cli --test cases -- oracle/`
Expected: PASS, with a case count matching the classes covered. Confirm the count printed is nonzero and equals what you authored — a filter matching nothing is the failure mode spec §9 names.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/tests/cases/oracle/classifier_ground_truth.toml
git commit -m "test(oracle): prove the classifier before trusting it on 84 entries"
```

---

## Task 7: Oracle cases — Tier 1

**Files:**
- Create: `crates/kali_cli/tests/cases/oracle/tier1.toml`

**Interfaces:**
- Consumes: `StepKind::Oracle` (Task 5), the ground truth (Task 6).
- Produces: measured verdicts for every Tier-1 entry, consumed by Task 10.

- [ ] **Step 1: Enumerate the entries**

```bash
awk '/^## Tier 1/,/^## Tier 2/' docs/superpowers/followups/kali-silent-miscompile-register.md \
  | grep '^### R-'
```

Every id printed needs **two** cases — module scope and in-function — because `sweep-common.md`'s binding method rule states top-level and in-function are different programs in kali, and several known defects appear at one scope only.

- [ ] **Step 2: Author the cases**

For each entry, read its §2 body, take its **minimal repro verbatim**, and write two cases. Do not invent a repro; the register's own is the measurement's provenance. Template:

```toml
[source]
"r01_module.js" = """<the register's module-scope repro, verbatim>
"""
"r01_function.js" = """<the register's in-function repro, verbatim>
"""

[[case]]
name = "r01_default_parameter_module_scope"
kind = "oracle"
register_entry = "R-01"
program = "r01_module.js"
verdict = "fail_closed"
rationale = """R-01, module scope. The register records FAIL-CLOSED as of 2026-07-24 (`62d786e74`): E5506 "default parameter is not supported", all forms, no truncation.

Expected verdict set from the register's recorded status. If this case fails, the entry has MOVED since that measurement -- which is the finding, not a bug in the case. Record the measured class and carry it into the §0.2 regeneration."""
```

Set each `verdict` to the register's **currently recorded** status, translated into a class. That makes a moved entry a red test carrying the old and new class in its message, which is exactly the signal Task 10 needs.

- [ ] **Step 3: Run the tier-1 cases**

Run: `cargo test -p kali_cli --test cases -- oracle/tier1`
Expected: a mix of pass and fail. **Failures here are data, not defects.** For each failure, record `(entry, scope, expected, measured)` in a scratch file for Task 10.

- [ ] **Step 4: Reconcile every failure**

For each failing case, verify the measured class by hand:

```bash
cargo build -p kali_cli
cd "$(mktemp -d)" && cp /workspace/<the-source> . && /workspace/target/debug/kali run <file>; echo "kali exit=$?"
node <file>; echo "node exit=$?"
```

Then update the case's `verdict` to the measured class and extend its rationale with the re-measurement: the date, the commit (`git rev-parse --short HEAD`), the node version, and the old class it superseded. Never change a verdict without recording what it was.

- [ ] **Step 5: Run again to green**

Run: `cargo test -p kali_cli --test cases -- oracle/tier1`
Expected: PASS, with a case count equal to twice the number of Tier-1 entries (less any entry where scope is genuinely moot, which the file's header comment must name and justify).

- [ ] **Step 6: Commit**

```bash
git add crates/kali_cli/tests/cases/oracle/tier1.toml
git commit -m "test(oracle): Tier 1 re-measured at HEAD against node v26.7.0"
```

---

## Task 8: Oracle cases — Tier 2

**Files:**
- Create: `crates/kali_cli/tests/cases/oracle/tier2.toml`

**Interfaces:**
- Consumes: `StepKind::Oracle` (Task 5).
- Produces: measured verdicts for every Tier-2 entry, consumed by Task 10.

Tier 2 holds the entries §0.1's amendment named as the likely frontier — R-10, R-13, R-14 — so these verdicts carry the most weight in the final ranking.

- [ ] **Step 1: Enumerate the entries**

```bash
awk '/^## Tier 2/,/^## Tier 3/' docs/superpowers/followups/kali-silent-miscompile-register.md \
  | grep '^### R-'
```

- [ ] **Step 2: Author, run, reconcile, and re-run**

Follow Task 7 steps 2–5 exactly, writing to `tier2.toml` and filtering with `oracle/tier2`. Two entries need care:

- **R-21** already moved: §0.2 records SILENT, but the absent-field lane fails closed `E5506 unknown field 'b' on fixed-shape object` at `64438bf0ef`. Expect a reconciliation here and record it.
- **R-06** is recorded as three sub-lanes (declarator-init fixed, `R-06-R2` reassignment silent, `R-06-R3` arrays silent). Give each sub-lane its own case with `register_entry = "R-06"`, and name the lane in the case `name` and `rationale`.

- [ ] **Step 3: Commit**

```bash
git add crates/kali_cli/tests/cases/oracle/tier2.toml
git commit -m "test(oracle): Tier 2 re-measured -- the frontier candidates get real verdicts"
```

---

## Task 9: Oracle cases — Tiers 3 and 4

**Files:**
- Create: `crates/kali_cli/tests/cases/oracle/tier3.toml`
- Create: `crates/kali_cli/tests/cases/oracle/tier4.toml`

**Interfaces:**
- Consumes: `StepKind::Oracle` (Task 5).
- Produces: measured verdicts for every Tier-3 and Tier-4 entry, consumed by Task 10.

- [ ] **Step 1: Enumerate both tiers**

```bash
awk '/^## Tier 3/,/^## Tier 4/' docs/superpowers/followups/kali-silent-miscompile-register.md | grep '^### R-'
awk '/^## Tier 4/,0' docs/superpowers/followups/kali-silent-miscompile-register.md | grep '^### R-'
```

- [ ] **Step 2: Author, run, reconcile, and re-run each file**

Follow Task 7 steps 2–5, writing to `tier3.toml` and `tier4.toml` and filtering with `oracle/tier3` and `oracle/tier4`.

Tier-4 entries are rendering-only, so several are distinguished purely by *how* a value prints (R-30 booleans as `1`/`0`, R-31 `console.log` of an object, R-32 exponential notation). Their repros must print through the exact sink the register names — a case that concatenates where the register logged directly measures a different lane and will disagree with §0.2 for the wrong reason.

- [ ] **Step 3: Run the whole oracle directory**

Run: `cargo test -p kali_cli --test cases -- oracle/`
Expected: PASS. Note the total case count.

- [ ] **Step 4: Verify coverage is complete**

```bash
grep -h '^register_entry' crates/kali_cli/tests/cases/oracle/*.toml \
  | sed 's/.*"\(R-[0-9]*\)".*/\1/' | sort -u > /tmp/covered.txt
grep '^### R-' docs/superpowers/followups/kali-silent-miscompile-register.md \
  | sed 's/^### \(R-[0-9]*\):.*/\1/' | sort -u > /tmp/expected.txt
diff /tmp/expected.txt /tmp/covered.txt && echo "COVERAGE COMPLETE"
```

Expected: `COVERAGE COMPLETE`. Any entry in `expected` but not `covered` needs a case before proceeding.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_cli/tests/cases/oracle/tier3.toml crates/kali_cli/tests/cases/oracle/tier4.toml
git commit -m "test(oracle): Tiers 3 and 4 re-measured -- every register entry now has a live verdict"
```

---

## Task 10: Regenerate register §0.2

**Files:**
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md` (§0.2 only)

**Interfaces:**
- Consumes: the measured verdicts in `crates/kali_cli/tests/cases/oracle/*.toml` (Tasks 7–9).
- Produces: a §0.2 whose every row is backed by a live case, consumed by Task 15.

- [ ] **Step 1: Extract the measured table**

```bash
python3 - <<'PY' > /tmp/measured.md
import re, pathlib
rows = {}
for path in sorted(pathlib.Path("crates/kali_cli/tests/cases/oracle").glob("tier*.toml")):
    text = path.read_text()
    for block in text.split("[[case]]")[1:]:
        entry = re.search(r'^register_entry = "([^"]+)"', block, re.M)
        verdict = re.search(r'^verdict = "([^"]+)"', block, re.M)
        name = re.search(r'^name = "([^"]+)"', block, re.M)
        if entry and verdict and name:
            rows.setdefault(entry.group(1), []).append((name.group(1), verdict.group(1)))
for entry in sorted(rows, key=lambda e: int(e.split("-")[1])):
    lanes = "; ".join(f"{n} = **{v.upper()}**" for n, v in rows[entry])
    print(f"| {entry} | {lanes} |")
PY
cat /tmp/measured.md
```

- [ ] **Step 2: Rewrite §0.2's header**

Replace the `### 0.2 Current status of every register entry` heading block's preamble with one that states the new provenance. Keep the existing verdict-vocabulary paragraph; add above the table:

```markdown
**Regenerated 2026-08-15.** Every row below is produced from the oracle cases in
`crates/kali_cli/tests/cases/oracle/`, measured at commit `<HEAD>` against
`node <version>`. A row is not prose a reader must re-derive: it is the verdict
a live case asserts, and a change of class is a red test. The prior table was
dated 2026-07-24 / `62d786e74` and had been stale for weeks — see §1 of
`docs/superpowers/specs/2026-08-15-blast-radius-ranking-design.md`.

Verdict classes are the classifier's, defined in
`crates/kali_blast_radius/src/verdict.rs`: `FIXED`, `SILENT`, `FAIL_CLOSED`,
`FL_INTERNAL`, `ACCEPTS_INVALID`, `BOTH_REJECT`, `TIMEOUT`, `NONDETERMINISTIC`.
```

Fill `<HEAD>` from `git rev-parse --short HEAD` and `<version>` from `node --version`.

- [ ] **Step 3: Replace the rows**

Replace the table body with `/tmp/measured.md`'s rows, preserving each row's existing explanatory note where it is still accurate. **Where a class changed, say so in the row** rather than silently overwriting — follow the register's own convention of striking rather than deleting:

```markdown
| R-21 no `undefined` value | **FAIL_CLOSED** (absent-field lane) / **SILENT** (other lanes) | ~~SILENT, all forms~~ — the absent-field lane moved to `E5506 unknown field` by `64438bf0ef`; §0.2 recorded SILENT until this regeneration. |
```

- [ ] **Step 4: Add the supersession note to §0**

At the end of §0's preamble (after the "`62d786e74` is a named baseline" paragraph), add:

```markdown
**Regeneration 2026-08-15.** §0.2 is no longer a hand-maintained table. It is
generated from the oracle cases under `crates/kali_cli/tests/cases/oracle/`,
which assert a derived verdict class and therefore fail when an entry moves.
Where this section's prose and §0.2 disagree about a class, **§0.2 wins** — it
is measured and this prose is not.
```

- [ ] **Step 5: Verify the table matches the cases**

Re-run the extraction from Step 1 and diff its output against the table you wrote. They must agree exactly. A row that does not appear in the extraction is a row with no case behind it.

- [ ] **Step 6: Commit**

```bash
git add docs/superpowers/followups/kali-silent-miscompile-register.md
git commit -m "docs(register): §0.2 regenerated from live cases, not re-derived by hand"
```

---

## Task 11: Corpus anchor

**Files:**
- Create: `tools/blast-radius/corpus/anchor/*.js`
- Create: `tools/blast-radius/corpus/manifest.json`
- Create: `crates/kali_blast_radius/src/manifest.rs`
- Test: `crates/kali_blast_radius/src/manifest_tests.rs`
- Modify: `crates/kali_blast_radius/src/lib.rs`

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces: `Manifest { corpus_hash: String, files: Vec<ManifestFile> }`,
  `ManifestFile { path: String, stratum: String, sha256: String }`,
  `parse_manifest(json: &str) -> Result<Manifest, String>`,
  `corpus_hash(files: &[ManifestFile]) -> String`,
  `verify_manifest(root: &std::path::Path, manifest: &Manifest) -> Result<(), String>`.

- [ ] **Step 1: Extract the anchor programs**

The six CLBG programs and the `imperative_core_runtime.rs` programs live as inline Rust string literals. Extract each into `tools/blast-radius/corpus/anchor/<name>.js`, verbatim:

```bash
ls crates/kali_cli/tests/clbg_*_runtime.rs
grep -n 'r#"' crates/kali_cli/tests/clbg_nbody_runtime.rs
```

For each program: copy the literal's contents exactly, with no reformatting. Name the file after the test function that held it. Verify each extraction round-trips:

```bash
node tools/blast-radius/corpus/anchor/<name>.js
```

A program that node refuses was mis-extracted — fix the extraction, do not edit the program.

- [ ] **Step 2: Write the failing manifest test**

```rust
// crates/kali_blast_radius/src/manifest_tests.rs
use super::*;

#[test]
fn the_corpus_hash_is_order_independent_and_content_sensitive() {
    let a = ManifestFile { path: "b.js".into(), stratum: "anchor".into(), sha256: "22".into() };
    let b = ManifestFile { path: "a.js".into(), stratum: "anchor".into(), sha256: "11".into() };
    let forward = corpus_hash(&[a.clone(), b.clone()]);
    let reversed = corpus_hash(&[b.clone(), a.clone()]);
    assert_eq!(forward, reversed, "hash must not depend on listing order");

    let changed = ManifestFile { sha256: "33".into(), ..a };
    assert_ne!(
        forward,
        corpus_hash(&[changed, b]),
        "a changed file must change the corpus hash"
    );
}

#[test]
fn verify_rejects_a_file_whose_content_changed() {
    let dir = std::env::temp_dir().join("blast-radius-manifest-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("anchor")).expect("mkdir");
    std::fs::write(dir.join("anchor/x.js"), "console.log(1);\n").expect("write");

    let good = ManifestFile {
        path: "anchor/x.js".into(),
        stratum: "anchor".into(),
        sha256: sha256_of("console.log(1);\n".as_bytes()),
    };
    let manifest = Manifest { corpus_hash: corpus_hash(&[good.clone()]), files: vec![good] };
    verify_manifest(&dir, &manifest).expect("an unmodified corpus verifies");

    std::fs::write(dir.join("anchor/x.js"), "console.log(2);\n").expect("write");
    let error = verify_manifest(&dir, &manifest).expect_err("a modified corpus must not verify");
    assert!(error.contains("anchor/x.js"), "error names the file: {error}");
}

#[test]
fn verify_rejects_an_untracked_file_in_the_corpus() {
    // A file present on disk but absent from the manifest would be counted by
    // the counter while the frozen hash still looked unchanged -- the exact
    // post-hoc corpus edit the freeze rule exists to prevent.
    let dir = std::env::temp_dir().join("blast-radius-untracked-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("anchor")).expect("mkdir");
    std::fs::write(dir.join("anchor/x.js"), "console.log(1);\n").expect("write");
    std::fs::write(dir.join("anchor/sneaky.js"), "console.log(2);\n").expect("write");

    let tracked = ManifestFile {
        path: "anchor/x.js".into(),
        stratum: "anchor".into(),
        sha256: sha256_of("console.log(1);\n".as_bytes()),
    };
    let manifest = Manifest { corpus_hash: corpus_hash(&[tracked.clone()]), files: vec![tracked] };
    let error = verify_manifest(&dir, &manifest).expect_err("an untracked file must not verify");
    assert!(error.contains("sneaky.js"), "error names the file: {error}");
}

#[test]
fn the_shipped_corpus_matches_its_manifest() {
    let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/blast-radius/corpus"));
    let text = std::fs::read_to_string(root.join("manifest.json")).expect("manifest is readable");
    let manifest = parse_manifest(&text).expect("manifest parses");
    assert!(!manifest.files.is_empty(), "an empty manifest must not verify as frozen");
    assert_eq!(
        manifest.corpus_hash,
        corpus_hash(&manifest.files),
        "the recorded corpus hash does not match its own file list"
    );
    verify_manifest(root, &manifest).expect("the shipped corpus matches its manifest");
}
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `cargo test -p kali_blast_radius manifest`
Expected: FAIL — `parse_manifest` not found.

- [ ] **Step 4: Implement the manifest**

```rust
// crates/kali_blast_radius/src/manifest.rs
//! The frozen corpus manifest.
//!
//! The freeze is what makes the ranking falsifiable. Without it, any desired
//! answer can be produced by adding or removing corpus programs after seeing
//! the scores, and no reader could detect it from the result. The published
//! ranking carries the corpus hash, so it pins exactly what was measured.

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestFile {
    /// Corpus-root-relative, e.g. `anchor/nbody.js`.
    pub path: String,
    /// `anchor` or `extension`. Accept rates are reported per stratum and
    /// never pooled: the anchor is passing tests, so a pooled rate would
    /// inherit its ~100% and mean nothing.
    pub stratum: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub corpus_hash: String,
    pub files: Vec<ManifestFile>,
}

pub fn sha256_of(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// A hash over the whole file list, independent of listing order so a manifest
/// reordering is not mistaken for a corpus change.
pub fn corpus_hash(files: &[ManifestFile]) -> String {
    let mut lines: Vec<String> = files
        .iter()
        .map(|file| format!("{} {} {}", file.stratum, file.path, file.sha256))
        .collect();
    lines.sort();
    sha256_of(lines.join("\n").as_bytes())
}

pub fn parse_manifest(json: &str) -> Result<Manifest, String> {
    serde_json::from_str(json).map_err(|error| format!("manifest is not valid json: {error}"))
}

/// Every manifest file exists with the recorded hash, and no `.js` file under
/// `root` is missing from the manifest.
///
/// Both directions are required. Checking only the recorded files would let an
/// untracked program be added to the corpus and counted while the frozen hash
/// still verified.
pub fn verify_manifest(root: &Path, manifest: &Manifest) -> Result<(), String> {
    for file in &manifest.files {
        let full = root.join(&file.path);
        let bytes = std::fs::read(&full)
            .map_err(|error| format!("manifest lists `{}`, which cannot be read: {error}", file.path))?;
        let actual = sha256_of(&bytes);
        if actual != file.sha256 {
            return Err(format!(
                "`{}` has changed since the freeze: manifest {}, on disk {}",
                file.path, file.sha256, actual
            ));
        }
    }

    let mut found = Vec::new();
    collect_js(root, root, &mut found)?;
    for path in found {
        if !manifest.files.iter().any(|file| file.path == path) {
            return Err(format!(
                "`{path}` is in the corpus directory but not in the manifest -- the corpus was \
                 edited after the freeze"
            ));
        }
    }
    Ok(())
}

fn collect_js(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|error| format!("cannot read {}: {error}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read entry: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_js(root, &path, out)?;
        } else if path.extension().is_some_and(|extension| extension == "js") {
            let relative = path
                .strip_prefix(root)
                .map_err(|error| format!("path escapes corpus root: {error}"))?;
            out.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod manifest_tests;
```

Add to `crates/kali_blast_radius/src/lib.rs`:

```rust
mod manifest;
pub use manifest::{corpus_hash, parse_manifest, sha256_of, verify_manifest, Manifest, ManifestFile};
```

- [ ] **Step 5: Generate the manifest**

```bash
python3 - <<'PY'
import hashlib, json, pathlib
root = pathlib.Path("tools/blast-radius/corpus")
files = []
for path in sorted(root.rglob("*.js")):
    rel = path.relative_to(root).as_posix()
    files.append({
        "path": rel,
        "stratum": rel.split("/")[0],
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
    })
lines = sorted(f"{f['stratum']} {f['path']} {f['sha256']}" for f in files)
digest = hashlib.sha256("\n".join(lines).encode()).hexdigest()
(root / "manifest.json").write_text(
    json.dumps({"corpus_hash": digest, "files": files}, indent=2) + "\n"
)
print(f"{len(files)} files, corpus_hash={digest}")
PY
```

- [ ] **Step 6: Run the tests**

Run: `cargo test -p kali_blast_radius`
Expected: PASS, including `the_shipped_corpus_matches_its_manifest`.

- [ ] **Step 7: Commit**

```bash
git add tools/blast-radius/corpus crates/kali_blast_radius
git commit -m "feat(blast-radius): the anchor corpus, extracted verbatim and frozen"
```

---

## Task 12: Corpus extension

**Files:**
- Create: `tools/blast-radius/corpus/extension/*.js`
- Create: `tools/blast-radius/corpus/README.md`
- Modify: `tools/blast-radius/corpus/manifest.json`

**Interfaces:**
- Consumes: the manifest machinery (Task 11).
- Produces: the frozen extension stratum, consumed by Tasks 13–14.

- [ ] **Step 1: Write the curation criteria first**

Create `tools/blast-radius/corpus/README.md`:

```markdown
# The blast-radius corpus

Two strata, never pooled.

- `anchor/` — the six CLBG programs and the `imperative_core_runtime.rs`
  programs, extracted verbatim from their inline Rust string fixtures. These
  are programs the project already committed to compiling, each with an
  end-to-end design behind it. They are accepted at essentially 100% *by
  construction* — they are passing tests — so their accept rate carries no
  information and is reported separately for exactly that reason.
- `extension/` — programs curated for this measurement.

## The curation rule

A program earns its place because it is what someone would plausibly write to
do a job kali targets. **Never because kali compiles it.**

This is load-bearing. If curation filtered on acceptance, the corpus would
exclude exactly the constructs the SILENT register entries trigger on, every
reachable frequency would be measured over a population selected for already
working, and the scores would be circular. Curation is independent of
measurement; reachability is applied afterwards and reported separately.

## The freeze

`manifest.json` is committed before the counter runs for record. Neither the
corpus nor `../predicates.json` may be adjusted after scores are visible. The
published ranking carries the corpus hash so a reader can tell exactly what was
measured. See the design spec §4.3.
```

- [ ] **Step 2: Curate the extension**

Write programs into `tools/blast-radius/corpus/extension/`. Each is a complete, runnable program doing a plausible job — a text transformation, a numeric simulation step, a small data reshaping, an argv-driven utility — written the way someone would write it, **without consulting what kali accepts**.

Target at least 25 programs. Coverage matters more than count: read `predicates.json` and make sure each countable predicate has a realistic chance of appearing somewhere, without contriving programs around the predicates. If a predicate would only appear in a contrived program, that is a finding about the predicate — note it in the README, do not manufacture a program.

Verify each runs under node:

```bash
for f in tools/blast-radius/corpus/extension/*.js; do
  node "$f" >/dev/null 2>&1 || echo "NODE REJECTS: $f"
done
```

Every program must run clean under node. A program node refuses measures nothing.

- [ ] **Step 3: Regenerate and verify the manifest**

Re-run the generator from Task 11 Step 5, then:

Run: `cargo test -p kali_blast_radius manifest`
Expected: PASS, with the file count now including the extension.

- [ ] **Step 4: Commit — this is the freeze**

```bash
git add tools/blast-radius/corpus
git commit -m "feat(blast-radius): the extension corpus, curated by intent and frozen"
```

After this commit, the corpus and `predicates.json` are frozen. Any later change to either must be a separate, explicitly-justified commit that says why.

---

## Task 13: The acorn counter and the accept table

**Files:**
- Create: `tools/blast-radius/package.json`, `tools/blast-radius/package-lock.json`
- Create: `tools/blast-radius/matchers.mjs`
- Create: `tools/blast-radius/matchers.test.mjs`
- Create: `tools/blast-radius/count.mjs`
- Create: `tools/blast-radius/accepts.mjs`
- Create: `tools/blast-radius/README.md`

**Interfaces:**
- Consumes: `predicates.json` (Task 2), the frozen corpus (Tasks 11–12).
- Produces: `tools/blast-radius/counts.json`
  (`{ corpusHash, nodeVersion, entries: [{ id, raw, reachable }] }`) and
  `tools/blast-radius/accepts.json`
  (`{ corpusHash, programs: [{ path, stratum, accepted }] }`), both consumed by Task 14.

- [ ] **Step 1: Pin acorn**

```bash
cd tools/blast-radius
npm init -y
npm install --save-exact acorn acorn-walk
```

Confirm `package.json` records exact versions with no `^`, and that `package-lock.json` exists. Both are committed.

- [ ] **Step 2: Write the failing matcher tests**

```javascript
// tools/blast-radius/matchers.test.mjs
import assert from "node:assert/strict";
import test from "node:test";
import { countAll, MATCHERS } from "./matchers.mjs";

test("computedMemberNonLiteralKey counts variable keys and not literal ones", () => {
  const src = `
    var o = {a: 1};
    var k = "a";
    console.log(o[k]);      // counts
    console.log(o["a"]);    // literal key, does not count
    console.log(o.a);       // not computed, does not count
    console.log(o[k + ""]); // non-literal expression, counts
  `;
  assert.equal(countAll(src).computedMemberNonLiteralKey, 2);
});

test("shadowingBlockDeclaration needs an enclosing binding of the same name", () => {
  const shadows = `var x = 1; { let x = 2; } console.log(x);`;
  const distinct = `var x = 1; { let y = 2; } console.log(x);`;
  assert.equal(countAll(shadows).shadowingBlockDeclaration, 1);
  assert.equal(countAll(distinct).shadowingBlockDeclaration, 0);
});

test("memberReadOnCallResult counts reads on a call, not calls on a member", () => {
  const src = `
    function f() { return [1, 2]; }
    console.log(f()[0]);   // counts
    console.log(f().a);    // counts
    console.log([1,2][0]); // literal receiver, does not count
    o.m();                 // a call ON a member, not a read on a call
  `;
  assert.equal(countAll(src).memberReadOnCallResult, 2);
});

test("a known-answer file counts exactly what it declares", () => {
  // Every matcher must return 0 on a program containing none of its shapes.
  const empty = `console.log("hello");`;
  const counts = countAll(empty);
  for (const name of Object.keys(MATCHERS)) {
    assert.equal(counts[name], 0, `${name} found a shape in a program with none`);
  }
});

test("a syntax error is thrown, never silently counted as zero", () => {
  assert.throws(() => countAll(`function ( {`), /parse/i);
});
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cd tools/blast-radius && node --test`
Expected: FAIL — `matchers.mjs` does not exist.

- [ ] **Step 4: Implement the matchers**

```javascript
// tools/blast-radius/matchers.mjs
//
// One matcher per countable predicate in predicates.json. The matcher name here
// MUST equal the `matcher` field there; count.mjs checks that and refuses to run
// if they disagree.
//
// acorn, not kali_parser. Counting the constructs kali miscompiles with kali's
// own parser is the confounded-instrument trap sweep-common.md rule 3 exists to
// prevent, and R-49 is the proof it is not hypothetical: parse_switch_statement
// silently reparented every post-switch statement for weeks with the suite green.

import * as acorn from "acorn";
import * as walk from "acorn-walk";

const PARSE_OPTIONS = { ecmaVersion: 2024, sourceType: "script", allowReturnOutsideFunction: false };

function parse(source) {
  try {
    return acorn.parse(source, PARSE_OPTIONS);
  } catch (cause) {
    // Never swallow this into a zero count: a file that fails to parse would
    // otherwise report "this construct does not appear here", which is a
    // measurement it did not make.
    throw new Error(`parse failed: ${cause.message}`, { cause });
  }
}

/** Names bound by a declaration node, for the shadowing matcher. */
function declaredNames(node, out) {
  if (!node) return out;
  if (node.type === "Identifier") out.add(node.name);
  else if (node.type === "ObjectPattern") node.properties.forEach((p) => declaredNames(p.value ?? p.argument, out));
  else if (node.type === "ArrayPattern") node.elements.forEach((e) => declaredNames(e, out));
  else if (node.type === "AssignmentPattern") declaredNames(node.left, out);
  else if (node.type === "RestElement") declaredNames(node.argument, out);
  return out;
}

export const MATCHERS = {
  // R-13: computed member access whose key expression is not a literal.
  computedMemberNonLiteralKey(ast) {
    let count = 0;
    walk.simple(ast, {
      MemberExpression(node) {
        if (node.computed && node.property.type !== "Literal") count += 1;
      },
    });
    return count;
  },

  // R-10: a let/const declaration in a nested block whose name is also bound
  // in an enclosing scope.
  shadowingBlockDeclaration(ast) {
    let count = 0;
    const walkScoped = (node, outer) => {
      if (!node || typeof node.type !== "string") return;
      let scope = outer;
      if (node.type === "BlockStatement" || node.type === "Program") {
        const here = new Set();
        for (const statement of node.body) {
          if (statement.type !== "VariableDeclaration") continue;
          for (const declarator of statement.declarations) {
            const names = declaredNames(declarator.id, new Set());
            for (const name of names) {
              if (statement.kind !== "var" && outer.has(name)) count += 1;
              here.add(name);
            }
          }
        }
        scope = new Set([...outer, ...here]);
      }
      for (const key of Object.keys(node)) {
        const child = node[key];
        if (Array.isArray(child)) child.forEach((each) => walkScoped(each, scope));
        else if (child && typeof child.type === "string") walkScoped(child, scope);
      }
    };
    walkScoped(ast, new Set());
    return count;
  },

  // R-14: a member or computed read applied directly to a call's result.
  memberReadOnCallResult(ast) {
    let count = 0;
    walk.simple(ast, {
      MemberExpression(node) {
        if (node.object.type === "CallExpression") count += 1;
      },
    });
    return count;
  },
};

/** Every matcher's count for one source string. */
export function countAll(source) {
  const ast = parse(source);
  const out = {};
  for (const [name, matcher] of Object.entries(MATCHERS)) {
    out[name] = matcher(ast);
  }
  return out;
}
```

Add one matcher per `countable` record in `predicates.json`, each with a test in `matchers.test.mjs` following the positive-and-negative shape above. The three written out here are the worked examples; the catalogue is the list of what must exist.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cd tools/blast-radius && node --test`
Expected: PASS.

- [ ] **Step 6: Write the accept table generator**

```javascript
// tools/blast-radius/accepts.mjs
//
// Reachability (design spec §6): per corpus program, binary -- does
// `kali check` exit 0? Occurrences in rejected programs score zero, because a
// defect kali fails closed on does no damage.

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const ROOT = path.dirname(new URL(import.meta.url).pathname);
const CORPUS = path.join(ROOT, "corpus");
const KALI = process.env.KALI_BIN ?? path.join(ROOT, "../../target/debug/kali");

const manifest = JSON.parse(fs.readFileSync(path.join(CORPUS, "manifest.json"), "utf8"));
if (manifest.files.length === 0) {
  throw new Error("the manifest is empty -- refusing to emit an accept table over nothing");
}

const programs = manifest.files.map((file) => {
  let accepted = false;
  try {
    execFileSync(KALI, ["check", path.join(CORPUS, file.path)], { stdio: "pipe" });
    accepted = true;
  } catch {
    accepted = false;
  }
  return { path: file.path, stratum: file.stratum, accepted };
});

const rates = {};
for (const program of programs) {
  const bucket = (rates[program.stratum] ??= { accepted: 0, total: 0 });
  bucket.total += 1;
  if (program.accepted) bucket.accepted += 1;
}

fs.writeFileSync(
  path.join(ROOT, "accepts.json"),
  `${JSON.stringify({ corpusHash: manifest.corpus_hash, programs }, null, 2)}\n`,
);
// Rates are printed per stratum and NEVER pooled: the anchor is passing tests,
// so a pooled rate would inherit its ~100% and mean nothing.
for (const [stratum, bucket] of Object.entries(rates)) {
  console.log(`${stratum}: ${bucket.accepted}/${bucket.total} accepted`);
}
```

- [ ] **Step 7: Write the counter**

```javascript
// tools/blast-radius/count.mjs
//
// Emits raw and reachable counts per register entry. Both are published: a
// reader can then see how much the reachability gate moved each entry instead
// of taking the gated number on faith.

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { countAll, MATCHERS } from "./matchers.mjs";

const ROOT = path.dirname(new URL(import.meta.url).pathname);
const CORPUS = path.join(ROOT, "corpus");

const manifest = JSON.parse(fs.readFileSync(path.join(CORPUS, "manifest.json"), "utf8"));
const catalogue = JSON.parse(fs.readFileSync(path.join(ROOT, "predicates.json"), "utf8"));
const accepts = JSON.parse(fs.readFileSync(path.join(ROOT, "accepts.json"), "utf8"));

if (accepts.corpusHash !== manifest.corpus_hash) {
  throw new Error(
    `accepts.json was generated against corpus ${accepts.corpusHash} but the manifest is ` +
      `${manifest.corpus_hash} -- re-run accepts.mjs`,
  );
}
if (manifest.files.length === 0) {
  throw new Error("the manifest is empty -- refusing to emit counts over nothing");
}

// The catalogue and the matcher module must agree, in both directions. A
// catalogue naming a matcher that does not exist would silently contribute
// nothing; a matcher with no catalogue record would be counted for no entry.
const countable = catalogue.entries.filter((entry) => entry.kind === "countable");
for (const entry of countable) {
  if (!(entry.matcher in MATCHERS)) {
    throw new Error(`predicates.json names matcher \`${entry.matcher}\` (${entry.id}), which matchers.mjs does not export`);
  }
}
for (const name of Object.keys(MATCHERS)) {
  if (!countable.some((entry) => entry.matcher === name)) {
    throw new Error(`matchers.mjs exports \`${name}\`, which no catalogue record names`);
  }
}

const acceptedPaths = new Set(accepts.programs.filter((p) => p.accepted).map((p) => p.path));
const raw = Object.fromEntries(Object.keys(MATCHERS).map((name) => [name, 0]));
const reachable = { ...raw };

for (const file of manifest.files) {
  const source = fs.readFileSync(path.join(CORPUS, file.path), "utf8");
  const counts = countAll(source);
  for (const [name, value] of Object.entries(counts)) {
    raw[name] += value;
    if (acceptedPaths.has(file.path)) reachable[name] += value;
  }
}

const entries = catalogue.entries.map((entry) =>
  entry.kind === "countable"
    ? { id: entry.id, raw: raw[entry.matcher], reachable: reachable[entry.matcher] }
    : { id: entry.id, raw: null, reachable: null },
);

const nodeVersion = execFileSync(process.execPath, ["--version"], { encoding: "utf8" }).trim();
fs.writeFileSync(
  path.join(ROOT, "counts.json"),
  `${JSON.stringify({ corpusHash: manifest.corpus_hash, nodeVersion, entries }, null, 2)}\n`,
);
console.log(`counted ${manifest.files.length} programs, ${countable.length} countable predicates`);
```

- [ ] **Step 8: Run both tools for record**

```bash
cargo build -p kali_cli
cd tools/blast-radius
node accepts.mjs
node count.mjs
```

Expected: per-stratum accept rates printed, `accepts.json` and `counts.json` written. **Record the extension's accept rate — it is a headline finding in its own right (spec §9).** If it is near zero, publish that; do not widen the corpus.

- [ ] **Step 9: Write the tool README**

Create `tools/blast-radius/README.md` documenting: the two commands in order (`accepts.mjs` before `count.mjs`, since the counter consumes the accept table), that `counts.json` and `accepts.json` are committed outputs, and that **the counter is not wired into CI** because doing so would touch a do-not-modify file — recorded, not worked around, alongside `docs/superpowers/followups/test-binary-consolidation-determinism-lane.md`.

- [ ] **Step 10: Commit**

```bash
git add tools/blast-radius
git commit -m "feat(blast-radius): count triggering constructs with acorn, gated by reachability"
```

---

## Task 14: Scoring and Pareto banding

**Files:**
- Create: `crates/kali_blast_radius/src/score.rs`
- Test: `crates/kali_blast_radius/src/score_tests.rs`
- Modify: `crates/kali_blast_radius/src/lib.rs`

**Interfaces:**
- Consumes: `RegisterEntry` (Task 1), `counts.json` (Task 13).
- Produces: `Cluster { name: String, entries: Vec<String>, tier: u8, reachable: Option<u64> }`,
  `aggregate(entries: &[ScoredEntry], clusters: &[(String, Vec<String>)]) -> Vec<Cluster>`,
  `dominates(a: &Cluster, b: &Cluster) -> bool`,
  `band(clusters: &[Cluster]) -> Vec<Vec<Cluster>>`,
  `ScoredEntry { id: String, tier: u8, reachable: Option<u64> }`.

Tier 1 is the *worst*, so a lower tier number dominates. `reachable: None` is `UNCOUNTABLE`.

- [ ] **Step 1: Write the failing test**

```rust
// crates/kali_blast_radius/src/score_tests.rs
use super::*;

fn cluster(name: &str, tier: u8, reachable: Option<u64>) -> Cluster {
    Cluster { name: name.into(), entries: vec![], tier, reachable }
}

#[test]
fn a_worse_tier_at_equal_frequency_dominates() {
    let worse = cluster("a", 1, Some(10));
    let better = cluster("b", 2, Some(10));
    assert!(dominates(&worse, &better));
    assert!(!dominates(&better, &worse));
}

#[test]
fn a_higher_frequency_at_equal_tier_dominates() {
    assert!(dominates(&cluster("a", 2, Some(50)), &cluster("b", 2, Some(10))));
}

#[test]
fn neither_dominates_when_each_wins_one_axis() {
    // This is the case a total order would have to break with an invented
    // weight. Both land in the same band instead.
    let a = cluster("a", 1, Some(2));
    let b = cluster("b", 2, Some(90));
    assert!(!dominates(&a, &b));
    assert!(!dominates(&b, &a));
}

#[test]
fn an_identical_pair_does_not_dominate_either_way() {
    let a = cluster("a", 2, Some(10));
    let b = cluster("b", 2, Some(10));
    assert!(!dominates(&a, &b));
    assert!(!dominates(&b, &a));
}

#[test]
fn uncountable_clusters_never_participate_in_dominance() {
    let counted = cluster("a", 1, Some(100));
    let unknown = cluster("b", 1, None);
    assert!(!dominates(&counted, &unknown));
    assert!(!dominates(&unknown, &counted));
}

#[test]
fn banding_peels_successive_pareto_frontiers() {
    let clusters = vec![
        cluster("top", 1, Some(100)),
        cluster("wide", 2, Some(90)),
        cluster("mid", 2, Some(50)),
        cluster("low", 3, Some(1)),
    ];
    let bands = band(&clusters);
    assert_eq!(bands.len(), 3, "expected three frontiers, got {bands:?}");
    assert_eq!(bands[0].iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), ["top", "wide"]);
    assert_eq!(bands[1].iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), ["mid"]);
    assert_eq!(bands[2].iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), ["low"]);
}

#[test]
fn banding_an_empty_input_yields_no_bands() {
    assert!(band(&[]).is_empty());
}

#[test]
fn banding_all_uncountable_yields_one_band() {
    // With no frequency axis, nothing dominates anything, so every cluster is
    // on the first frontier. That is the honest outcome, not a ranking.
    let clusters = vec![cluster("a", 1, None), cluster("b", 3, None)];
    let bands = band(&clusters);
    assert_eq!(bands.len(), 1);
    assert_eq!(bands[0].len(), 2);
}

#[test]
fn aggregation_takes_the_worst_tier_and_sums_reachable() {
    let entries = vec![
        ScoredEntry { id: "R-02".into(), tier: 1, reachable: Some(4) },
        ScoredEntry { id: "R-03".into(), tier: 2, reachable: Some(6) },
    ];
    let clusters = vec![("call-lowering-choke".to_string(), vec!["R-02".to_string(), "R-03".to_string()])];
    let aggregated = aggregate(&entries, &clusters);
    assert_eq!(aggregated.len(), 1);
    assert_eq!(aggregated[0].tier, 1, "the worst tier in the cluster wins");
    assert_eq!(aggregated[0].reachable, Some(10));
}

#[test]
fn a_cluster_with_any_uncountable_member_is_uncountable() {
    // Summing over a partially-counted cluster would publish a number smaller
    // than the truth while looking complete.
    let entries = vec![
        ScoredEntry { id: "R-15".into(), tier: 2, reachable: Some(4) },
        ScoredEntry { id: "R-16".into(), tier: 2, reachable: None },
    ];
    let clusters = vec![("string-repr".to_string(), vec!["R-15".to_string(), "R-16".to_string()])];
    assert_eq!(aggregate(&entries, &clusters)[0].reachable, None);
}

#[test]
fn aggregation_rejects_a_cluster_naming_an_unscored_entry() {
    let entries = vec![ScoredEntry { id: "R-02".into(), tier: 1, reachable: Some(4) }];
    let clusters = vec![("c".to_string(), vec!["R-02".to_string(), "R-99".to_string()])];
    let result = std::panic::catch_unwind(|| aggregate(&entries, &clusters));
    assert!(result.is_err(), "an unscored member must not be silently dropped");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p kali_blast_radius score`
Expected: FAIL — `Cluster` not found.

- [ ] **Step 3: Write the implementation**

```rust
// crates/kali_blast_radius/src/score.rs
//! Cluster aggregation and Pareto banding.
//!
//! Bands, not a total order. A strict 1-through-N ordering over these entries
//! would need a weight relating tier to frequency, and no measurement here
//! justifies one: pick 1000/100/10/1 and tier always wins, pick 4/3/2/1 and
//! frequency does. The constants, not the data, would decide the answer. A
//! Pareto frontier needs no constant at all -- see the design spec §3.3, §8.2.

/// One register entry with its two axes. `reachable: None` is UNCOUNTABLE.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoredEntry {
    pub id: String,
    pub tier: u8,
    pub reachable: Option<u64>,
}

/// A root cause, which is the unit a fix actually ships in: R-02, R-03 and
/// R-05 were all closed by one allowlist at the call-lowering choke, and R-49
/// and R-54 both live in `parse_switch_statement`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cluster {
    pub name: String,
    pub entries: Vec<String>,
    /// The WORST tier among members (lowest number).
    pub tier: u8,
    /// Sum over members, or `None` if any member is uncountable.
    pub reachable: Option<u64>,
}

/// Roll entries up into clusters.
///
/// # Panics
///
/// If a cluster names an entry with no score. Dropping it silently would
/// under-count that cluster while the output still looked complete.
pub fn aggregate(entries: &[ScoredEntry], clusters: &[(String, Vec<String>)]) -> Vec<Cluster> {
    clusters
        .iter()
        .map(|(name, members)| {
            let scored: Vec<&ScoredEntry> = members
                .iter()
                .map(|id| {
                    entries
                        .iter()
                        .find(|entry| &entry.id == id)
                        .unwrap_or_else(|| panic!("cluster `{name}` names `{id}`, which has no score"))
                })
                .collect();
            let tier = scored.iter().map(|entry| entry.tier).min().unwrap_or(u8::MAX);
            // Any uncountable member makes the whole cluster uncountable: a sum
            // over a partially-counted cluster is smaller than the truth while
            // looking complete.
            let reachable = scored
                .iter()
                .try_fold(0u64, |total, entry| entry.reachable.map(|value| total + value));
            Cluster { name: name.clone(), entries: members.clone(), tier, reachable }
        })
        .collect()
}

/// Does `a` dominate `b`? Tier 1 is the worst, so a LOWER tier dominates.
///
/// An uncountable cluster never dominates and is never dominated: with no
/// frequency, there is no comparison to make, and inventing one is the failure
/// this design exists to avoid.
pub fn dominates(a: &Cluster, b: &Cluster) -> bool {
    let (Some(a_reachable), Some(b_reachable)) = (a.reachable, b.reachable) else {
        return false;
    };
    let no_worse = a.tier <= b.tier && a_reachable >= b_reachable;
    let strictly_better = a.tier < b.tier || a_reachable > b_reachable;
    no_worse && strictly_better
}

/// Successive Pareto frontiers: band 1 is the non-dominated set, band 2 is the
/// non-dominated set of what remains, and so on. Order within a band is the
/// input order, which callers should make deterministic before calling.
pub fn band(clusters: &[Cluster]) -> Vec<Vec<Cluster>> {
    let mut remaining: Vec<Cluster> = clusters.to_vec();
    let mut bands = Vec::new();
    while !remaining.is_empty() {
        let frontier: Vec<Cluster> = remaining
            .iter()
            .filter(|candidate| !remaining.iter().any(|other| dominates(other, candidate)))
            .cloned()
            .collect();
        if frontier.is_empty() {
            // Unreachable for a strict dominance relation (it is irreflexive
            // and transitive, so a finite non-empty set always has a maximal
            // element), but a silent infinite loop is not an acceptable
            // failure mode if that ever stops holding.
            panic!("no cluster is non-dominated among {} remaining -- dominance is not strict", remaining.len());
        }
        remaining.retain(|cluster| !frontier.iter().any(|kept| kept.name == cluster.name));
        bands.push(frontier);
    }
    bands
}

#[cfg(test)]
#[path = "score_tests.rs"]
mod score_tests;
```

Add to `crates/kali_blast_radius/src/lib.rs`:

```rust
mod score;
pub use score::{aggregate, band, dominates, Cluster, ScoredEntry};
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p kali_blast_radius`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_blast_radius
git commit -m "feat(blast-radius): Pareto bands -- no weights, no order the data cannot support"
```

---

## Task 15: Publish the ranking

**Files:**
- Create: `docs/superpowers/followups/blast-radius-ranking.md`
- Modify: `docs/superpowers/followups/kali-silent-miscompile-register.md` (§0.1)

**Interfaces:**
- Consumes: `counts.json` (Task 13), the regenerated §0.2 (Task 10), `aggregate`/`band` (Task 14).
- Produces: the ranking, which is what the next fix-project cites.

- [ ] **Step 1: Assign clusters**

Read §2 and §3 of the register and write down each **currently-`SILENT`** entry's root cause. The register already names clusters (G1 parser, G2 call-lowering choke, G3) — use its names where they exist and add new ones only where they do not. Record the assignment in the ranking document as a table, so it can be argued with.

- [ ] **Step 2: Compute the bands**

Filter `counts.json` to entries whose regenerated §0.2 verdict is `SILENT`, build `ScoredEntry` values from those counts plus their tiers, run `aggregate` then `band`, and record the result. Only `SILENT` entries enter — `FIXED`, `FAIL_CLOSED` and `BOTH_REJECT` are not damage (spec §8.1).

- [ ] **Step 3: Write the ranking document**

`docs/superpowers/followups/blast-radius-ranking.md`, containing, in order:

1. **What this is and what it supersedes** — §0.1's "the frontier is unranked" statement, and the corpus hash, HEAD commit, and node version the measurement was taken at.
2. **The bands** — band 1 first, each cluster with its tier, reachable frequency, and member entries.
3. **The per-entry table** — id, tier, raw count, reachable count, verdict, cluster. Generated. This is what lets a reader re-derive a different composition if they disagree with the banding.
4. **The uncountable list** — each entry with its written reason, banded on tier alone and never merged into the numeric bands.
5. **The accept rates**, per stratum, never pooled.
6. **Commentary** — clearly marked as authored, not generated.

Every number in sections 2–5 comes from a committed table. Do not retype a count by hand.

- [ ] **Step 4: Supersede §0.1**

Add to the register's §0.1, after the 2026-07-29 amendment, following the register's own convention — state the supersession, strike rather than delete:

```markdown
**Amendment 2026-08-15 — THE FRONTIER IS NOW RANKED.** The 2026-07-29 amendment's
point 2 said the frontier was unranked and named the three things that would
settle it: an operational definition of blast radius, a re-measurement of every
§0.2 verdict, and then a ranking. All three are done.

- The definition is `docs/superpowers/specs/2026-08-15-blast-radius-ranking-design.md`
  §3: the pair `(tier, reachable_frequency)`, where frequency is counted only
  over corpus programs kali accepts.
- §0.2 is regenerated from live cases under `crates/kali_cli/tests/cases/oracle/`.
- The ranking is `docs/superpowers/followups/blast-radius-ranking.md`.

~~the frontier is unranked, and it is somewhere in {R-10, R-13, R-14, R-31, and
the rest of the pre-existing SILENT set}~~ — superseded. The measured band 1 is
in the ranking document; that document, not this paragraph, is authoritative.
```

- [ ] **Step 5: Verify the whole gate is green**

```bash
cargo build --workspace
cargo test --workspace
bash scripts/test-gate.sh
```

Expected: all green. Report the actual output — if a lane fails, say so with the output rather than summarising it as passing.

- [ ] **Step 6: Commit**

```bash
git add docs/superpowers/followups/blast-radius-ranking.md docs/superpowers/followups/kali-silent-miscompile-register.md
git commit -m "docs: the frontier, ranked -- measured bands supersede the unranked amendment"
```

---

## Self-Review Notes

**Spec coverage.** §3 definition → Tasks 1–2, 14. §4 corpus → Tasks 11–12. §5 counter → Task 13. §6 reachability → Task 13 (`accepts.mjs`). §7 oracle + classifier → Tasks 3, 5. §7.1 `E4xxx` → Task 4. §8.1 pipeline/SILENT filter → Task 15. §8.2 clusters + Pareto → Task 14. §8.3 outputs → Tasks 10, 13, 15. §9 failure modes → Task 5 (timeout, nondeterminism, missing-program), Task 13 (empty-manifest refusal), Task 2 Step 6 (the stop gate). §10 instrument testing → Tasks 6, 13 Step 2, 14. §11 constraints → Global Constraints, Task 13 Step 9. §12 sequencing risk → Task 2 Step 6. §13 sequencing → task order.

**Known judgment call.** Tasks 7–9 do not write out all ~84 fixtures. Each case's source must be the register entry's own repro, copied verbatim from §2 — writing invented sources into this plan would substitute the plan author's guess for the register's evidence. The tasks therefore give the enumeration command, the exact template, a worked example, and the reconciliation procedure, and Task 9 Step 4 gates on complete coverage mechanically.
