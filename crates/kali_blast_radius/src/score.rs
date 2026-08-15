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
                        .unwrap_or_else(|| {
                            panic!("cluster `{name}` names `{id}`, which has no score")
                        })
                })
                .collect();
            let tier = scored
                .iter()
                .map(|entry| entry.tier)
                .min()
                .unwrap_or(u8::MAX);
            // Any uncountable member makes the whole cluster uncountable: a sum
            // over a partially-counted cluster is smaller than the truth while
            // looking complete.
            let reachable = scored.iter().try_fold(0u64, |total, entry| {
                entry.reachable.map(|value| total + value)
            });
            Cluster {
                name: name.clone(),
                entries: members.clone(),
                tier,
                reachable,
            }
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
            panic!(
                "no cluster is non-dominated among {} remaining -- dominance is not strict",
                remaining.len()
            );
        }
        remaining.retain(|cluster| !frontier.iter().any(|kept| kept.name == cluster.name));
        bands.push(frontier);
    }
    bands
}

#[cfg(test)]
#[path = "score_tests.rs"]
mod score_tests;
