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
