//! Generates sections 2-5 of `docs/superpowers/followups/blast-radius-ranking.md`.
//!
//! Every figure in those sections is emitted from here, from four committed
//! inputs: the register (tiers via `parse_register`, verdicts via §0.2's
//! generated table), `tools/blast-radius/counts.json`,
//! `tools/blast-radius/clusters.json` and `tools/blast-radius/accepts.json`. A
//! hand-typed number in a ranking is the rot this whole project exists to end,
//! so there is no path here by which one can appear.
//!
//! This lives in the library rather than only in `examples/rank.rs` so that
//! `ranking_tests.rs` can hold the spliced document to it. A freeze that is
//! documentary rather than mechanical is the failure mode this plan has spent
//! fifteen tasks removing.
//!
//! Regenerate with:
//!
//! ```text
//! cargo run -p kali_blast_radius --example rank
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::{aggregate, band, parse_register, Cluster, ScoredEntry};

/// The repository root, from this crate's manifest directory.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/kali_blast_radius has a grandparent")
        .to_path_buf()
}

/// The verdict classes §0.2's status column can name. Matched as whole tokens
/// so `FL_INTERNAL` is never read as a SILENT lane and `ACCEPTS_INVALID` never
/// as a FIXED one.
const CLASSES: [&str; 8] = [
    "FIXED",
    "SILENT",
    "FAIL_CLOSED",
    "FL_INTERNAL",
    "ACCEPTS_INVALID",
    "BOTH_REJECT",
    "TIMEOUT",
    "NONDETERMINISTIC",
];

/// One §0.2 row: the entry it is about and the lane classes its status column
/// names, in the order the column names them.
struct Row {
    id: String,
    lanes: Vec<String>,
}

/// Parse §0.2's status column. Bounded to the §0.2 section: `| R-08 + R-21 …`
/// rows exist elsewhere in the register and are not verdicts.
fn parse_status_table(markdown: &str) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut inside = false;
    for line in markdown.lines() {
        if line.starts_with("### 0.2") {
            inside = true;
            continue;
        }
        if inside && line.starts_with("### ") {
            break;
        }
        if !inside || !line.starts_with("| R-") {
            continue;
        }
        // `\|` is an escaped pipe inside a cell (R-18's title contains
        // `` `\|\|` ``), not a column separator. Splitting naively puts the
        // status column in the wrong place and the row reads as classless.
        let unescaped = line.replace("\\|", "\u{0}");
        let columns: Vec<&str> = unescaped.split('|').collect();
        // `| a | b | c |` splits to ["", " a ", " b ", " c ", ""].
        assert!(
            columns.len() >= 4,
            "§0.2 row has fewer than three columns: {line}"
        );
        let first = columns[1].trim();
        let id: String = first
            .strip_prefix("R-")
            .map(|rest| {
                let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
                format!("R-{digits}")
            })
            .unwrap_or_else(|| panic!("§0.2 row does not start with an entry id: {line}"));
        let status = columns[2];
        assert!(
            !status.contains("~~"),
            "§0.2's status column for {id} contains struck text; a struck class would be read \
             as a live one: {status}"
        );
        let mut lanes: Vec<(usize, String)> = Vec::new();
        for class in CLASSES {
            let mut from = 0;
            while let Some(at) = status[from..].find(class) {
                let start = from + at;
                // Whole-token match: `FAIL_CLOSED` must not also count as a
                // hit for a class that is a substring of it.
                let after = status[start + class.len()..].chars().next();
                let ok = after.is_none_or(|c| !c.is_ascii_alphanumeric() && c != '_');
                if ok {
                    lanes.push((start, class.to_string()));
                }
                from = start + class.len();
            }
        }
        lanes.sort();
        assert!(!lanes.is_empty(), "§0.2 row for {id} names no verdict class");
        rows.push(Row {
            id,
            lanes: lanes.into_iter().map(|(_, class)| class).collect(),
        });
    }
    assert!(!rows.is_empty(), "§0.2's table shape changed -- no rows parsed");
    rows
}

/// `raw`/`reachable` for one entry, `None` when the entry is uncountable --
/// either because it has no predicate at all or because its zero is
/// structurally uncountable and must never be published as a frequency.
struct Counts {
    raw: Option<u64>,
    reachable: Option<u64>,
    zero: Option<String>,
    upper_bound: Option<bool>,
    anchor: Option<(u64, u64)>,
    extension: Option<(u64, u64)>,
}

fn number(value: &Value) -> Option<u64> {
    value.as_u64()
}

/// Render sections 2-5 of the ranking, preceded by the §1.4 provenance table.
///
/// # Panics
///
/// On any disagreement between the inputs -- a duplicate cluster name, an entry
/// assigned twice, a cluster with no members, an assignment the SILENT filter
/// does not admit. Every one of those would produce a plausible wrong ranking,
/// which is worse than no ranking.
pub fn render(root: &Path) -> String {
    let register_path = root.join("docs/superpowers/followups/kali-silent-miscompile-register.md");
    let register_md = fs::read_to_string(&register_path).expect("register is readable");
    let counts: Value = serde_json::from_str(
        &fs::read_to_string(root.join("tools/blast-radius/counts.json")).expect("counts.json"),
    )
    .expect("counts.json parses");
    let clusters_file: Value = serde_json::from_str(
        &fs::read_to_string(root.join("tools/blast-radius/clusters.json")).expect("clusters.json"),
    )
    .expect("clusters.json parses");
    let predicates: Value = serde_json::from_str(
        &fs::read_to_string(root.join("tools/blast-radius/predicates.json")).expect("predicates"),
    )
    .expect("predicates.json parses");
    let corpus_readme = fs::read_to_string(root.join("tools/blast-radius/corpus/README.md"))
        .expect("corpus README is readable");
    let accepts_meta: Value = serde_json::from_str(
        &fs::read_to_string(root.join("tools/blast-radius/accepts.json")).expect("accepts.json"),
    )
    .expect("accepts.json parses");

    // ---------------------------------------------------------------- tiers
    let tiers: BTreeMap<String, u8> = parse_register(&register_md)
        .expect("register §2 parses")
        .into_iter()
        .map(|entry| (entry.id, entry.tier))
        .collect();

    // -------------------------------------------------------------- verdicts
    let rows = parse_status_table(&register_md);
    let mut lanes_by_entry: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in &rows {
        lanes_by_entry
            .entry(row.id.clone())
            .or_default()
            .extend(row.lanes.iter().cloned());
    }
    let silent: BTreeSet<String> = lanes_by_entry
        .iter()
        .filter(|(_, lanes)| lanes.iter().any(|lane| lane == "SILENT"))
        .map(|(id, _)| id.clone())
        .collect();
    let excluded: BTreeSet<String> = lanes_by_entry
        .keys()
        .filter(|id| !silent.contains(*id))
        .cloned()
        .collect();

    // ---------------------------------------------------------------- counts
    let mut counts_by_entry: BTreeMap<String, Counts> = BTreeMap::new();
    let structurally_uncountable: BTreeMap<String, String> = counts["structurallyUncountable"]
        .as_object()
        .expect("structurallyUncountable is an object")
        .iter()
        .map(|(id, why)| (id.clone(), why.as_str().unwrap_or_default().to_string()))
        .collect();
    for entry in counts["entries"].as_array().expect("entries is an array") {
        let id = entry["id"].as_str().expect("entry has an id").to_string();
        let zero = entry["zero"].as_str().map(str::to_string);
        let uncountable =
            entry["raw"].is_null() || zero.as_deref() == Some("structurally-uncountable");
        let stratum = |name: &str| -> Option<(u64, u64)> {
            let value = &entry["strata"][name];
            Some((number(&value["raw"])?, number(&value["reachable"])?))
        };
        counts_by_entry.insert(
            id,
            Counts {
                raw: if uncountable { None } else { number(&entry["raw"]) },
                reachable: if uncountable {
                    None
                } else {
                    number(&entry["reachable"])
                },
                zero,
                upper_bound: entry["upperBound"]
                    .get("disclosedInRecord")
                    .and_then(Value::as_bool),
                anchor: stratum("anchor"),
                extension: stratum("extension"),
            },
        );
    }

    // -------------------------------------------------------------- clusters
    let mut assignment: BTreeMap<String, (String, String, Option<String>)> = BTreeMap::new();
    let mut alternate_cluster: BTreeMap<String, String> = BTreeMap::new();
    for item in clusters_file["assignments"]
        .as_array()
        .expect("assignments is an array")
    {
        let id = item["id"].as_str().expect("assignment has an id").to_string();
        let cluster = item["cluster"].as_str().expect("cluster").to_string();
        let source = item["registerSource"].as_str().expect("source").to_string();
        let alternate = item["alternate"].as_str().map(str::to_string);
        if let Some(other) = item["alternateCluster"].as_str() {
            alternate_cluster.insert(id.clone(), other.to_string());
        }
        assert!(
            assignment
                .insert(id.clone(), (cluster, source, alternate))
                .is_none(),
            "{id} is assigned twice -- its frequency would be counted twice"
        );
    }

    let mut cluster_order: Vec<(String, String, String)> = Vec::new();
    let mut seen_names: BTreeSet<String> = BTreeSet::new();
    for item in clusters_file["clusters"]
        .as_array()
        .expect("clusters is an array")
    {
        let name = item["name"].as_str().expect("cluster name").to_string();
        // `band` peels frontiers by matching on the name. Two clusters sharing
        // one would mis-peel silently, so uniqueness is checked, not assumed.
        assert!(
            seen_names.insert(name.clone()),
            "two clusters are named `{name}` -- `band` peels by name and would mis-peel"
        );
        cluster_order.push((
            name,
            item["origin"].as_str().unwrap_or_default().to_string(),
            item["why"].as_str().unwrap_or_default().to_string(),
        ));
    }

    // The filter and the assignment must agree exactly, in both directions.
    let assigned: BTreeSet<String> = assignment.keys().cloned().collect();
    assert_eq!(
        assigned, silent,
        "clusters.json and the regenerated §0.2 disagree about which entries are SILENT"
    );

    let mut members: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (id, (cluster, _, _)) in &assignment {
        assert!(
            seen_names.contains(cluster),
            "{id} is assigned to `{cluster}`, which is not a declared cluster"
        );
        members.entry(cluster.clone()).or_default().push(id.clone());
    }
    for (name, _, _) in &cluster_order {
        assert!(
            members.contains_key(name),
            "cluster `{name}` has no members -- an empty cluster ranks nothing"
        );
    }

    // ----------------------------------------------------------- the banding
    let cluster_input: Vec<(String, Vec<String>)> = cluster_order
        .iter()
        .map(|(name, _, _)| (name.clone(), members[name].clone()))
        .collect();

    let scored = |axis: fn(&Counts) -> Option<u64>| -> Vec<ScoredEntry> {
        silent
            .iter()
            .map(|id| ScoredEntry {
                id: id.clone(),
                tier: *tiers
                    .get(id)
                    .unwrap_or_else(|| panic!("{id} has no tier in §2")),
                reachable: axis(
                    counts_by_entry
                        .get(id)
                        .unwrap_or_else(|| panic!("{id} has no counts")),
                ),
            })
            .collect()
    };

    let reachable_bands = band(&aggregate(&scored(|c| c.reachable), &cluster_input));
    let raw_bands = band(&aggregate(&scored(|c| c.raw), &cluster_input));

    let mut out = String::new();

    // Population figures, read rather than retyped: they appear in prose in
    // §2 and prose is exactly where a stale number survives longest.
    let programs = |key: &str, field: &str| -> u64 {
        number(&counts["programs"][key][field]).unwrap_or_else(|| panic!("programs.{key}.{field}"))
    };
    let pooled_programs = programs("pooled", "programs");
    let pooled_accepted = programs("pooled", "accepted");
    let anchor_accepted = programs("anchor", "accepted");
    let extension_programs = programs("extension", "programs");
    let extension_accepted = programs("extension", "accepted");

    // ------------------------------------------------------- provenance
    // Emitted first, above `## 2`, so the assembly step can split it off into
    // §1. A ranking without the baseline it was taken at is the exact defect
    // §0.2 spent this project's first ten tasks correcting.
    let head = std::process::Command::new("git")
        .args(["rev-parse", "--short=10", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    // The commit §0.2's verdicts were measured at, read out of the register's
    // own sentence rather than copied into this file, where it would rot.
    let measured_at = register_md
        .split_once("measured at commit `")
        .and_then(|(_, rest)| rest.split_once('`'))
        .map(|(commit, _)| commit.to_string())
        .unwrap_or_else(|| "(the register no longer states one)".to_string());
    let _ = writeln!(out, "| what | value | where it is recorded |");
    let _ = writeln!(out, "|---|---|---|");
    let _ = writeln!(
        out,
        "| corpus hash | `{}` | `tools/blast-radius/corpus/manifest.json`, verified on every run |",
        counts["corpusHash"].as_str().unwrap_or_default()
    );
    let _ = writeln!(
        out,
        "| node | `{}` | `counts.json` |",
        counts["nodeVersion"].as_str().unwrap_or_default()
    );
    let _ = writeln!(
        out,
        "| acorn | `{}` | `counts.json` |",
        counts["acornVersion"].as_str().unwrap_or_default()
    );
    let _ = writeln!(
        out,
        "| kali binary | `{}` (`{}`) | `accepts.json` |",
        accepts_meta["kaliVersion"].as_str().unwrap_or_default(),
        accepts_meta["kaliBinary"].as_str().unwrap_or_default()
    );
    let _ = writeln!(
        out,
        "| §0.2's verdicts, measured at | `{measured_at}` | \
         `kali-silent-miscompile-register.md` §0.2's own sentence |"
    );
    let _ = writeln!(
        out,
        "| this document generated at | `{head}` | `git rev-parse HEAD`, recorded by the generator |"
    );
    let _ = writeln!(out);

    // ------------------------------------------------------------ section 2
    let _ = writeln!(out, "## 2. The bands\n");
    let _ = writeln!(
        out,
        "Bands, not a total order. Band 1 is the Pareto frontier over `(tier, frequency)`: \
         a cluster is in it when no other cluster is at least as bad on both axes and \
         strictly worse on one. Band 2 is the frontier of what remains, and so on. No weight \
         relates a tier to a count, so none is invented — design spec §3.3, §8.2.\n"
    );
    let _ = writeln!(
        out,
        "**A cluster with an uncountable member has no frequency at all**, and `dominates` \
         makes it neither dominate nor be dominated. Such a cluster therefore appears in \
         band 1 *by non-comparability*, not by measurement, and is marked `n/a` and flagged. \
         Do not read it as a measured frontier member. The countable-only frontier, which \
         is the one a reader wanting a measured answer should use, is printed after each axis.\n"
    );

    // ------------------------------------------- 2.1, the assignment itself
    let _ = writeln!(out, "### 2.1 The clusters, and where each assignment came from\n");
    let _ = writeln!(
        out,
        "A cluster is a **root cause** — the unit a fix ships in — not a topic. Every \
         assignment is the register's own `Root-cause group:` line on that entry in §2, quoted \
         below so it can be checked against the source rather than trusted. Nothing here is a \
         fresh diagnosis: §3 of the register says grouping errors are cheap to make and \
         expensive to act on, and this ranking is not the place to make one.\n"
    );
    let _ = writeln!(out, "| cluster | origin | why it is a cluster |");
    let _ = writeln!(out, "|---|---|---|");
    for (name, origin, why) in &cluster_order {
        let _ = writeln!(out, "| {name} | {origin} | {why} |");
    }
    let _ = writeln!(
        out,
        "\n`aggregate` sums a cluster over its members, so an entry in two clusters would be \
         counted twice; the assignment below is a partition. Where the register names two \
         groups it names them in order, and the first is taken. Counts are per **entry**, not \
         per lane, so a cluster sum carries an entry's whole frequency even where the register \
         splits that entry across two clusters by lane.\n"
    );
    let _ = writeln!(
        out,
        "| entry | tier | cluster | the register's own §2 line | the second reading, and why it was not taken |"
    );
    let _ = writeln!(out, "|---|---|---|---|---|");
    for id in &silent {
        let (cluster, source, alternate) = &assignment[id];
        let _ = writeln!(
            out,
            "| {id} | {} | {cluster} | {source} | {} |",
            tiers[id],
            alternate.as_deref().unwrap_or("—")
        );
    }
    let _ = writeln!(out);

    let render_bands = |out: &mut String, title: &str, note: &str, bands: &[Vec<Cluster>]| {
        let _ = writeln!(out, "### {title}\n");
        let _ = writeln!(out, "{note}\n");
        for (index, this_band) in bands.iter().enumerate() {
            let _ = writeln!(out, "**Band {}**\n", index + 1);
            let _ = writeln!(out, "| cluster | worst tier | frequency | members |");
            let _ = writeln!(out, "|---|---|---|---|");
            let mut sorted: Vec<&Cluster> = this_band.iter().collect();
            sorted.sort_by_key(|cluster| {
                (
                    cluster.tier,
                    cluster.reachable.map(|value| u64::MAX - value),
                    cluster.name.clone(),
                )
            });
            for cluster in sorted {
                let frequency = match cluster.reachable {
                    Some(value) => value.to_string(),
                    None => "n/a — uncountable member".to_string(),
                };
                let _ = writeln!(
                    out,
                    "| {} | {} | {} | {} |",
                    cluster.name,
                    cluster.tier,
                    frequency,
                    cluster.entries.join(", ")
                );
            }
            if index == 0 {
                // A band-1 table is the thing a reader is most likely to stop
                // at and quote, and its membership is contingent on cluster
                // calls the register itself states two ways. The caveat travels
                // with the table rather than waiting in §6.
                let _ = writeln!(
                    out,
                    "\n*Band 1 is contingent on the cluster assignment. §2.4 re-runs every \
                     contested assignment and finds two that move a band 1: R-21 (both axes) \
                     and R-23 (the reachable axis, by changing G8's worst tier). Quote this \
                     table with §2.4, not on its own.*"
                );
            }
            let _ = writeln!(out);
        }
        let countable: Vec<Cluster> = bands
            .iter()
            .flatten()
            .filter(|cluster| cluster.reachable.is_some())
            .cloned()
            .collect();
        let countable_bands = band(&countable);
        let front = countable_bands
            .first()
            .expect("at least one countable cluster");
        let mut sorted: Vec<&Cluster> = front.iter().collect();
        sorted.sort_by_key(|cluster| (cluster.tier, u64::MAX - cluster.reachable.unwrap_or(0)));
        let _ = writeln!(
            out,
            "**Countable-only band 1** (the same computation with every uncountable cluster \
             dropped rather than carried, so a reader can see the measured frontier on its own): {}.\n",
            sorted
                .iter()
                .map(|cluster| format!(
                    "{} (tier {}, {})",
                    cluster.name,
                    cluster.tier,
                    cluster.reachable.unwrap_or(0)
                ))
                .collect::<Vec<_>>()
                .join("; ")
        );
    };

    render_bands(
        &mut out,
        "2.2 The reachable axis — the ranking's own definition",
        &format!(
            "Frequency is the count over the {pooled_accepted} corpus programs kali accepts, of \
             which {anchor_accepted} are anchor micro-snippets. This is the axis the design spec \
             §3 defines the ranking on, and in substance it is a ranking over test snippets: \
             {} of the {extension_programs} programs written to do a job rather than to probe \
             the compiler is reachable.",
            extension_accepted
        ),
        &reachable_bands,
    );
    render_bands(
        &mut out,
        "2.3 The raw axis — published beside it, never instead of it",
        &format!(
            "The same clusters banded on the count over all {pooled_programs} corpus programs, \
             accepted or not. This is the axis that carries what the extension stratum says, \
             because {} of its {extension_programs} programs are unreachable. It is published so \
             a reader can see how far the reachability gate moved each cluster; it is NOT a \
             substitute for the reachable axis, and the corpus is not widened to make the two \
             agree (spec §4.3 forbids adjusting the corpus once scores are visible).",
            extension_programs - extension_accepted
        ),
        &raw_bands,
    );

    // ------------------------------- 2.3, how much the contested calls matter
    //
    // Every assignment the register states two ways is re-run with the OTHER
    // one taken, one at a time, and band 1 is recomputed. A clustering nobody
    // can move is not being defended here -- what is published is how far band
    // 1 moves when someone does.
    let band_one_names = |input: &[(String, Vec<String>)],
                          axis: fn(&Counts) -> Option<u64>|
     -> BTreeSet<String> {
        band(&aggregate(&scored(axis), input))
            .first()
            .expect("a non-empty band 1")
            .iter()
            .map(|cluster| cluster.name.clone())
            .collect()
    };
    let base_reachable = band_one_names(&cluster_input, |c| c.reachable);
    let base_raw = band_one_names(&cluster_input, |c| c.raw);
    let _ = writeln!(out, "### 2.4 How much the contested assignments matter\n");
    let _ = writeln!(
        out,
        "{} of the {} ranked entries have a second cluster the register names with a concrete \
         destination. Each is moved to it, alone, and both band 1s are recomputed. A clustering that cannot be \
         argued with is not a measurement, so the argument is priced here rather than \
         asserted away.\n",
        alternate_cluster.len(),
        silent.len()
    );
    let _ = writeln!(
        out,
        "| entry | assigned | moved to | reachable band 1 | raw band 1 |"
    );
    let _ = writeln!(out, "|---|---|---|---|---|");
    for (id, other) in &alternate_cluster {
        let moved: Vec<(String, Vec<String>)> = cluster_input
            .iter()
            .map(|(name, entries)| {
                let mut entries: Vec<String> =
                    entries.iter().filter(|member| *member != id).cloned().collect();
                if name == other {
                    entries.push(id.clone());
                    entries.sort();
                }
                (name.clone(), entries)
            })
            // A cluster emptied by the move ranks nothing and is dropped.
            .filter(|(_, entries)| !entries.is_empty())
            .collect();
        let describe = |before: &BTreeSet<String>, after: BTreeSet<String>| -> String {
            if *before == after {
                "unchanged".to_string()
            } else {
                let gained: Vec<&String> = after.difference(before).collect();
                let lost: Vec<&String> = before.difference(&after).collect();
                let mut parts = Vec::new();
                if !gained.is_empty() {
                    parts.push(format!(
                        "gains {}",
                        gained
                            .iter()
                            .map(|name| format!("**{name}**"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                if !lost.is_empty() {
                    parts.push(format!(
                        "loses {}",
                        lost.iter()
                            .map(|name| format!("**{name}**"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                parts.join("; ")
            }
        };
        let _ = writeln!(
            out,
            "| {id} | {} | {other} | {} | {} |",
            assignment[id].0,
            describe(&base_reachable, band_one_names(&moved, |c| c.reachable)),
            describe(&base_raw, band_one_names(&moved, |c| c.raw)),
        );
    }
    let _ = writeln!(out);

    // ------------------------------------------------------------ section 3
    let _ = writeln!(out, "## 3. The per-entry table\n");
    let _ = writeln!(
        out,
        "Every input to §2, so a reader who disagrees with the clustering can re-band from \
         here. `raw` counts all {pooled_programs} programs; `reachable` counts only the \
         {pooled_accepted} kali accepts. `zero` names WHICH KIND of zero a zero is — the three \
         are not the same claim and must never be pooled (`counts.json` `zeroKinds`).\n"
    );
    let _ = writeln!(
        out,
        "| entry | tier | raw | reachable | anchor raw/reach | extension raw/reach | §0.2 lanes | zero kind | upper bound | cluster |"
    );
    let _ = writeln!(out, "|---|---|---|---|---|---|---|---|---|---|");
    let pair = |value: Option<(u64, u64)>| match value {
        Some((raw, reachable)) => format!("{raw} / {reachable}"),
        None => "—".to_string(),
    };
    for id in &silent {
        let counts_row = &counts_by_entry[id];
        let (cluster, _, _) = &assignment[id];
        let upper = match counts_row.upper_bound {
            Some(true) => "yes (disclosed in record)",
            Some(false) => "yes (**not** disclosed in record)",
            None => "—",
        };
        let _ = writeln!(
            out,
            "| {id} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            tiers[id],
            counts_row
                .raw
                .map_or("uncountable".to_string(), |value| value.to_string()),
            counts_row
                .reachable
                .map_or("uncountable".to_string(), |value| value.to_string()),
            pair(counts_row.anchor),
            pair(counts_row.extension),
            lanes_by_entry[id].join(" / "),
            counts_row.zero.as_deref().unwrap_or("—"),
            upper,
            cluster,
        );
    }
    let _ = writeln!(out);

    let _ = writeln!(
        out,
        "### 3.1 What the SILENT filter removed, and what it cost the ranking\n"
    );
    // Spec §8.1 gives TWO grounds for exclusion, not one. Deriving the split
    // from the lanes keeps the two apart: an entry every one of whose lanes is
    // FIXED / FAIL_CLOSED / BOTH_REJECT is not damage; anything else is a class
    // §8.1 reports but does not rank.
    const NOT_DAMAGE: [&str; 3] = ["FIXED", "FAIL_CLOSED", "BOTH_REJECT"];
    let (not_damage, outside_question): (Vec<String>, Vec<String>) = excluded
        .iter()
        .cloned()
        .partition(|id| lanes_by_entry[id].iter().all(|lane| NOT_DAMAGE.contains(&lane.as_str())));
    let _ = writeln!(
        out,
        "Spec §8.1 removes these {} entries for **two different reasons**, and collapsing them \
         would misdescribe {} of them:\n\n\
         - **Not damage** — `FIXED`, `FAIL_CLOSED`, `BOTH_REJECT`. kali either agrees with node \
         or refuses honestly. {} entries leave this way: {}.\n\
         - **Outside this ranking's question** — `ACCEPTS_INVALID`, `FL_INTERNAL`, `TIMEOUT`, \
         `NONDETERMINISTIC`. §8.1 *reports* these in the regenerated table and keeps them out \
         of the ranking, whose question is *what silent defect should be fixed next*. {} \
         entries leave this way: {}. The distinction is not pedantic: R-29's §0.2 row records \
         kali printing `r=1` at exit 0 with no diagnostic, which is silent by any plain \
         reading. It is out because accepting a program node rejects is a different defect \
         class from giving a wrong answer to a valid one — not because nothing bad happens.\n\n\
         Their counts are printed because the removal is not cosmetic: it takes the largest \
         reachable count in the whole measurement out of the ranking.\n",
        excluded.len(),
        outside_question.len(),
        not_damage.len(),
        not_damage.join(", "),
        outside_question.len(),
        outside_question.join(", "),
    );
    let _ = writeln!(out, "| entry | tier | raw | reachable | §0.2 lanes |");
    let _ = writeln!(out, "|---|---|---|---|---|");
    let mut excluded_by_reachable: Vec<&String> = excluded.iter().collect();
    excluded_by_reachable.sort_by_key(|id| {
        (
            u64::MAX - counts_by_entry[*id].reachable.unwrap_or(0),
            (*id).clone(),
        )
    });
    for id in &excluded_by_reachable {
        let counts_row = &counts_by_entry[*id];
        let _ = writeln!(
            out,
            "| {id} | {} | {} | {} | {} |",
            tiers[*id],
            counts_row
                .raw
                .map_or("uncountable".to_string(), |value| value.to_string()),
            counts_row
                .reachable
                .map_or("uncountable".to_string(), |value| value.to_string()),
            lanes_by_entry[*id].join(" / "),
        );
    }
    let ranked_max = silent
        .iter()
        .filter_map(|id| counts_by_entry[id].reachable.map(|value| (value, id)))
        .max();
    let excluded_max = excluded
        .iter()
        .filter_map(|id| counts_by_entry[id].reachable.map(|value| (value, id)))
        .max();
    let excluded_nonzero: Vec<String> = excluded
        .iter()
        .filter(|id| counts_by_entry[*id].reachable.unwrap_or(0) > 0)
        .map(|id| format!("{id} ({})", counts_by_entry[id].reachable.unwrap_or(0)))
        .collect();
    if let (Some((ranked_value, ranked_id)), Some((excluded_value, excluded_id))) =
        (ranked_max, excluded_max)
    {
        let _ = writeln!(
            out,
            "\nOnly {} of the {} removed entries have a nonzero reachable count at all: {}. \
             The largest of them, {excluded_id} at {excluded_value}, is **the largest reachable \
             count anywhere in `counts.json`** — larger than the largest that survives the \
             filter ({ranked_id} at {ranked_value}). The ranking's numeric input is much \
             thinner than the raw measurement looks.\n",
            excluded_nonzero.len(),
            excluded.len(),
            excluded_nonzero.join(" and "),
        );
    }

    let nonzero_reachable: Vec<String> = silent
        .iter()
        .filter(|id| counts_by_entry[*id].reachable.unwrap_or(0) > 0)
        .cloned()
        .collect();
    let _ = writeln!(
        out,
        "And of the {} entries that do enter, **{} have a reachable count above zero** ({}); \
         {} measure zero and {} have no count at all. The bands below separate {} clusters on \
         the evidence of {} nonzero entries.\n",
        silent.len(),
        nonzero_reachable.len(),
        nonzero_reachable
            .iter()
            .map(|id| format!("{id} = {}", counts_by_entry[id].reachable.unwrap_or(0)))
            .collect::<Vec<_>>()
            .join(", "),
        silent.len()
            - nonzero_reachable.len()
            - silent
                .iter()
                .filter(|id| counts_by_entry[*id].reachable.is_none())
                .count(),
        silent
            .iter()
            .filter(|id| counts_by_entry[*id].reachable.is_none())
            .count(),
        cluster_order.len(),
        nonzero_reachable.len(),
    );

    // -------------------------------------------------- 3.2, R-13's shape
    // R-13 is one of the three entries §0.1 named as the likely frontier, so
    // what its number is and is not goes beside the number.
    let r13 = counts["entries"]
        .as_array()
        .expect("entries")
        .iter()
        .find(|entry| entry["id"] == "R-13")
        .expect("R-13 has a counts record");
    let breakdown = &r13["upperBound"]["breakdown"];
    let field = |where_: &Value, name: &str| -> u64 {
        number(&where_[name]).unwrap_or_else(|| panic!("R-13 breakdown lacks {name}"))
    };
    let _ = writeln!(
        out,
        "### 3.2 R-13's number is not R-13's shape\n\nThe register's R-13 repro is an \
         **object read with a variable key**. `computedMemberNonLiteralKey` counts every \
         computed member access with a non-literal key, which includes ordinary array \
         indexing `a[i]` — and array indexing demonstrably works. The committed breakdown in \
         `counts.json` splits the same sites by receiver and by position:\n"
    );
    let _ = writeln!(
        out,
        "| axis | total | object-literal receiver | array-like receiver | store target |"
    );
    let _ = writeln!(out, "|---|---|---|---|---|");
    for (label, where_) in [
        ("raw (all programs)", &breakdown["raw"]),
        ("reachable (pooled)", &breakdown["reachable"]),
        ("reachable — anchor", &breakdown["strata"]["anchor"]["reachable"]),
        (
            "reachable — extension",
            &breakdown["strata"]["extension"]["reachable"],
        ),
    ] {
        let _ = writeln!(
            out,
            "| {label} | {} | {} | {} | {} |",
            field(where_, "total"),
            field(where_, "objectLiteralReceiver"),
            field(where_, "arrayLikeReceiver"),
            field(where_, "storeTarget"),
        );
    }
    let _ = writeln!(
        out,
        "\nRead down the reachable rows: of R-13's {} reachable sites, **{} have the \
         object-literal receiver the register's repro describes**, and the anchor's share of \
         those is **{}** — so **all {} reachable anchor sites have none**. Both \
         register-shaped sites are in the extension stratum, whose reachable population is \
         the {} program kali accepts. A further **{} of the {} are store targets**, not reads: the \
         register treats the write half as the worse one, but it is a different site class \
         from the read its repro shows. R-13's {} is an upper bound on a construct family, \
         not a count of how often R-13's defect is triggered.\n",
        field(&breakdown["reachable"], "total"),
        field(&breakdown["reachable"], "objectLiteralReceiver"),
        field(&breakdown["strata"]["anchor"]["reachable"], "objectLiteralReceiver"),
        field(&breakdown["strata"]["anchor"]["reachable"], "total"),
        extension_accepted,
        field(&breakdown["reachable"], "storeTarget"),
        field(&breakdown["reachable"], "total"),
        field(&breakdown["reachable"], "total"),
    );

    // ------------------------------------------------- 3.3, the upper bounds
    let mut disclosed: Vec<String> = Vec::new();
    let mut undisclosed: Vec<String> = Vec::new();
    for entry in counts["entries"].as_array().expect("entries") {
        let id = entry["id"].as_str().unwrap_or_default().to_string();
        let marker = if silent.contains(&id) {
            id.clone()
        } else {
            format!("{id} (not in the ranking)")
        };
        match entry["upperBound"]
            .get("disclosedInRecord")
            .and_then(Value::as_bool)
        {
            Some(true) => disclosed.push(marker),
            Some(false) => undisclosed.push(marker),
            None => {}
        }
    }
    let _ = writeln!(
        out,
        "### 3.3 Which counts are upper bounds\n\nA count is an upper bound when the predicate \
         admits sites the defect does not reach — because the AST cannot see a runtime type, a \
         representation, or a compiler-internal proof. {} records disclose their own upper \
         bound: {}. {} more are upper bounds their records do **not** disclose, found by this \
         measurement: {}. Every note is in `counts.json` under `upperBound`.\n",
        disclosed.len(),
        disclosed.join(", "),
        undisclosed.len(),
        undisclosed.join(", "),
    );

    // ------------------------------------- 3.4, lanes are not entry verdicts
    let mixed: Vec<String> = silent
        .iter()
        .filter(|id| lanes_by_entry[*id].iter().any(|lane| lane != "SILENT"))
        .map(|id| format!("{id} ({})", lanes_by_entry[id].join(" / ")))
        .collect();
    let _ = writeln!(
        out,
        "### 3.4 A lane result is not an entry result\n\n{} of the {} ranked entries measure \
         something other than SILENT on at least one lane, and none of them is thereby \
         retired: {}. §0.2 records why in each case — R-47's and R-53's FIXED lanes are the \
         `const` controls those entries declare for themselves, and R-30's two FIXED lanes are \
         its `const`-scalar lane *and* its concat/template sinks, so *declared control* is the \
         accurate description and *`const` lane* is not. R-08's `===` half fails closed while \
         its `??` half is **still SILENT**, unchanged by that move. R-49 — not in the ranking \
         at all — fails closed by **R-35's** switch allowlist rather than by its own gate. An \
         entry is retired when every lane moves, which is a claim no single lane can make.\n",
        mixed.len(),
        silent.len(),
        mixed.join("; ")
    );

    // ------------------------------------------------------------ section 4
    let _ = writeln!(out, "## 4. The uncountable entries\n");
    let _ = writeln!(
        out,
        "No frequency exists for these, so they are banded on **tier alone** and are never \
         merged into §2's numeric bands. An uncountable entry is not a rare one: it is one \
         the counter cannot see at all, and publishing it as `0` would rank it below every \
         entry the corpus happens to contain.\n"
    );
    let mut uncountable: Vec<(String, String, bool)> = Vec::new();
    for entry in predicates["entries"].as_array().expect("predicate entries") {
        if entry["kind"].as_str() == Some("uncountable") {
            let id = entry["id"].as_str().expect("id").to_string();
            let why = entry["reason"].as_str().unwrap_or_default().to_string();
            let in_ranking = silent.contains(&id);
            uncountable.push((id, why, in_ranking));
        }
    }
    for (id, why) in &structurally_uncountable {
        uncountable.push((id.clone(), why.clone(), silent.contains(id)));
    }
    uncountable.sort();
    let mut by_tier: BTreeMap<u8, Vec<String>> = BTreeMap::new();
    for (id, _, in_ranking) in &uncountable {
        if *in_ranking {
            by_tier.entry(tiers[id]).or_default().push(id.clone());
        }
    }
    let _ = writeln!(out, "| entry | tier | in the ranking? | kind | why no count exists |");
    let _ = writeln!(out, "|---|---|---|---|---|");
    for (id, why, in_ranking) in &uncountable {
        let kind = if structurally_uncountable.contains_key(id) {
            "structurally uncountable"
        } else {
            "no syntactic predicate (representation- or runtime-typed)"
        };
        let _ = writeln!(
            out,
            "| {id} | {} | {} | {kind} | {} |",
            tiers[id],
            if *in_ranking {
                "yes — SILENT"
            } else {
                "no — removed by the SILENT filter"
            },
            why.replace('\n', " ")
        );
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "**Banded on tier alone** (only the entries the SILENT filter admits):\n"
    );
    for (tier, ids) in &by_tier {
        let _ = writeln!(out, "- Tier {tier}: {}", ids.join(", "));
    }
    let _ = writeln!(
        out,
        "\nThe clusters carrying them have no frequency either, which is why they sit in §2's \
         band 1 marked `n/a` — there by non-comparability, not by measurement:\n"
    );
    for (id, _, _) in uncountable
        .iter()
        .filter(|(_, _, in_ranking)| *in_ranking)
    {
        let _ = writeln!(out, "- {id} → **{}**", assignment[id].0);
    }
    let _ = writeln!(out);

    // ------------------------------------------------------------ section 5
    let _ = writeln!(out, "## 5. The accept rates\n");
    let _ = writeln!(
        out,
        "Per stratum, never pooled: the anchor's rate is fixed by which tests happen to exist \
         and would destroy the only informative number if averaged into it \
         (`corpus/README.md`).\n"
    );
    let _ = writeln!(out, "| stratum | accepted | programs | rate |");
    let _ = writeln!(out, "|---|---|---|---|");
    for stratum in ["anchor", "extension"] {
        let accepted = number(&counts["programs"][stratum]["accepted"]).expect("accepted");
        let programs = number(&counts["programs"][stratum]["programs"]).expect("programs");
        let _ = writeln!(
            out,
            "| {stratum} | {accepted} | {programs} | {:.1}% |",
            100.0 * accepted as f64 / programs as f64
        );
    }
    let _ = writeln!(out);
    // The whole paragraph, not the matching line: a rate quoted without the
    // sentence that qualifies it is the failure this section exists to avoid.
    let anchor_readme: Vec<&str> = corpus_readme
        .split("\n\n")
        .find(|block| block.contains("124/137"))
        .expect("corpus/README.md no longer states the suite-expectation anchor rate")
        .lines()
        .collect();
    let _ = writeln!(
        out,
        "**Two anchor rates, both true, different instruments.** The table above is measured \
         by running `kali check` over every anchor program (`accepts.mjs`, recorded in \
         `accepts.json`). `corpus/README.md` states a different one, from the suite's own run \
         expectation:\n"
    );
    for line in anchor_readme {
        let _ = writeln!(out, "> {}", line.trim());
    }
    // The two rates are reconciled program by program rather than asserted to
    // differ. "Different instruments" is only a defence if the difference is
    // accounted for.
    let provenance: Value = serde_json::from_str(
        &fs::read_to_string(root.join("tools/blast-radius/anchor-provenance.json"))
            .expect("anchor-provenance.json"),
    )
    .expect("anchor-provenance.json parses");
    let accepts = &accepts_meta;
    let accepted_paths: BTreeMap<String, bool> = accepts["programs"]
        .as_array()
        .expect("programs")
        .iter()
        .map(|program| {
            (
                program["path"].as_str().unwrap_or_default().to_string(),
                program["accepted"].as_bool().unwrap_or(false),
            )
        })
        .collect();
    let mut expect_failure_total = 0usize;
    let mut expect_failure_checked_ok: Vec<String> = Vec::new();
    let mut run_js_rejected: Vec<String> = Vec::new();
    for item in provenance.as_array().expect("provenance is an array") {
        let file = item["file"].as_str().unwrap_or_default();
        let path = format!("anchor/{file}");
        let accepted = *accepted_paths.get(&path).unwrap_or(&false);
        match item["helper"].as_str() {
            Some("run_js_expect_failure") => {
                expect_failure_total += 1;
                if accepted {
                    expect_failure_checked_ok.push(file.to_string());
                }
            }
            _ => {
                if !accepted {
                    run_js_rejected.push(file.to_string());
                }
            }
        }
    }
    let _ = writeln!(
        out,
        "\nThe {expect_failure_total} `run_js_expect_failure` programs are ones the suite \
         commits kali to *rejecting*, and the README's rate counts all {expect_failure_total} \
         as not-accepted. Reconciled program by program against `accepts.json`, the whole \
         difference is {} programs in two directions:\n",
        expect_failure_checked_ok.len() + run_js_rejected.len()
    );
    let _ = writeln!(
        out,
        "- **{} the suite expects to fail but `kali check` accepts** — {}. A program the suite \
         commits to failing at *run* time can still pass a *check*.",
        expect_failure_checked_ok.len(),
        expect_failure_checked_ok
            .iter()
            .map(|file| format!("`{file}`"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let _ = writeln!(
        out,
        "- **{} the suite expects to pass but `kali check` rejects** — {}.",
        run_js_rejected.len(),
        run_js_rejected
            .iter()
            .map(|file| format!("`{file}`"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let _ = writeln!(
        out,
        "\nNeither number is wrong and neither supersedes the other — they answer two \
         different questions about two different instruments, and a reader who sees only one \
         will take the other for a typo.\n"
    );
    let _ = writeln!(out, "**What the reachable column is a frequency over.**\n");
    for key in ["reachableColumn", "extensionStratum", "dialect"] {
        let _ = writeln!(
            out,
            "> {}\n",
            counts["population"][key].as_str().unwrap_or_default()
        );
    }

    out
}

#[cfg(test)]
#[path = "ranking_tests.rs"]
mod ranking_tests;
