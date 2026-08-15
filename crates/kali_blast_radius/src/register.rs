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
///
/// Three-valued on purpose. `Ok(None)` is "not an entry header at all", which
/// is every other line in the file. `Err` is "this line *is* an entry header
/// and its id is not one this parser can read" -- `### R-06-R2:`, say, whose
/// number parses as `06-R2` and fails the all-digits check.
///
/// It used to return `None` for both, and the entry VANISHED: no error, no
/// entry, a register that still parsed and a count that was quietly one short.
/// The register already writes `R-06-R2` / `R-06-R3` in its prose, so that
/// header shape is one naming convention away from being live -- and a silent
/// drop is the wrong failure mode to ship inside a silent-miscompile project.
fn entry_of_header(line: &str) -> Result<Option<(String, String)>, String> {
    let Some(rest) = line.strip_prefix("### R-") else {
        return Ok(None);
    };
    let Some((number, title)) = rest.split_once(':') else {
        return Ok(None);
    };
    if number.is_empty() || !number.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!(
            "`### R-` header has an id this parser cannot read: `{}` -- the number is `{number}`, \
             which is not all digits. Dropping it silently would take the entry out of every \
             count downstream with nothing red",
            line.trim()
        ));
    }
    Ok(Some((format!("R-{number}"), title.trim().to_string())))
}

pub fn parse_register(markdown: &str) -> Result<Vec<RegisterEntry>, String> {
    let mut entries: Vec<RegisterEntry> = Vec::new();
    let mut tier: Option<u8> = None;
    let mut in_section_2 = true;

    for line in markdown.lines() {
        if let Some(found) = tier_of_header(line) {
            tier = Some(found);
            in_section_2 = true;
            continue;
        }
        // If we see a non-tier `## ` heading after the tier section has started, we've left §2.
        // R-50 is filed in §7 because it is not a silent miscompile — kali exits nonzero
        // with a diagnostic — and tiering it would rank a fail-loudly defect as rendering-only
        // damage. The register's numbering note states that §2 holds 41 tier-ranked entries,
        // and R-50 is the sole `### R-` header outside §2's tier headings. Once a non-tier
        // `## ` heading appears after we've seen a tier header, subsequent `### R-` entries
        // are outside the tier table and must be skipped.
        if line.starts_with("## ") && tier.is_some() && tier_of_header(line).is_none() {
            in_section_2 = false;
            continue;
        }
        // Checked before the §2 bound, not after: a header this parser cannot
        // read is a defect wherever it sits, and deciding to ignore it because
        // of where it sits is the silent drop under another name.
        let Some((id, title)) = entry_of_header(line)? else {
            continue;
        };
        // Skip entries that appear outside §2 (after a non-tier `## ` heading).
        if !in_section_2 {
            continue;
        }
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
