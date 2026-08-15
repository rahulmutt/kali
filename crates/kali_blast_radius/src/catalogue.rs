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
    Countable {
        matcher: String,
        description: String,
    },
    Uncountable {
        reason: String,
    },
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
    let raw: RawCatalogue = serde_json::from_str(json)
        .map_err(|error| format!("catalogue is not valid json: {error}"))?;

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
                Predicate::Countable {
                    matcher,
                    description,
                }
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
        _ => Err(format!(
            "`{id}` has no `{field}` -- it must be written down, not left blank"
        )),
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
        let hits = catalogue
            .iter()
            .filter(|record| record.id == entry.id)
            .count();
        if hits == 0 {
            missing.push(&entry.id);
        } else if hits > 1 {
            return Err(format!(
                "`{}` has {hits} catalogue records -- expected exactly 1",
                entry.id
            ));
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
