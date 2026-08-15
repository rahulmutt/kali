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
