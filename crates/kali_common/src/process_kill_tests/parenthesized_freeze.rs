use super::*;

#[test]
fn test_process_kill_zero_probe_parenthesized_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = process_kill_zero_probe_parenthesized_frozen_callable_aliases();
    let source = process_kill_zero_probe_parenthesized_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    assert_eq!(
        aliases,
        &[
            r#"Object.freeze((process.kill))(0)"#,
            r#"Object.freeze((process.kill))(+0)"#,
            r#"Object.freeze((globalThis.process.kill))(0)"#,
            r#"Object.freeze((globalThis.process.kill))(+0)"#,
            r#"Object.freeze((process["kill"]))(0)"#,
            r#"Object.freeze((process["kill"]))(+0)"#,
            r#"Object.freeze((globalThis.process["kill"]))(0)"#,
            r#"Object.freeze((globalThis.process["kill"]))(+0)"#,
            r#"Object.freeze((globalThis["process"].kill))(0)"#,
            r#"Object.freeze((globalThis["process"].kill))(+0)"#,
            r#"Object.freeze((globalThis["process"]["kill"]))(0)"#,
            r#"Object.freeze((globalThis["process"]["kill"]))(+0)"#,
        ]
    );

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in parenthesized frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, expected);
}

#[test]
fn test_process_kill_zero_probe_parenthesized_receiver_freeze_inventory_source_composes_both_helpers(
) {
    let source = process_kill_zero_probe_parenthesized_receiver_freeze_inventory_source();
    let expected = format!(
        "{} {}",
        process_kill_zero_probe_parenthesized_receiver_freeze_source().trim_end(),
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_inventory_source().trim_end()
    );

    assert_eq!(source, expected);
}

#[test]
fn test_process_kill_zero_probe_parenthesized_receiver_freeze_inventory_source_is_prefix_free_and_single_sourced(
) {
    let inventory_source = process_kill_zero_probe_parenthesized_receiver_freeze_inventory_source();
    let prefix_source = late_process_control_prefix_source();
    let direct_source = process_kill_zero_probe_parenthesized_receiver_freeze_source();
    let bracket_source =
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_inventory_source();

    assert_eq!(
        inventory_source,
        format!("{} {}", direct_source.trim_end(), bracket_source.trim_end())
    );
    assert!(
        !inventory_source.contains(&prefix_source),
        "inventory: {inventory_source}"
    );
    assert_eq!(
        inventory_source.matches(direct_source.trim_end()).count(),
        1,
        "inventory: {inventory_source}"
    );
    assert_eq!(
        inventory_source.matches(bracket_source.trim_end()).count(),
        1,
        "inventory: {inventory_source}"
    );
}

#[test]
fn test_process_kill_zero_probe_parenthesized_receiver_freeze_inventory_source_does_not_include_late_process_control_prefix(
) {
    let inventory_source = process_kill_zero_probe_parenthesized_receiver_freeze_inventory_source();
    let prefix_source = late_process_control_prefix_source();

    assert!(
        !inventory_source.contains(&prefix_source),
        "inventory: {inventory_source}"
    );
}

#[test]
fn test_process_kill_zero_probe_parenthesized_receiver_freeze_bracket_source_lists_all_aliases_in_order(
) {
    let aliases = process_kill_zero_probe_parenthesized_receiver_freeze_bracket_aliases();
    let source = process_kill_zero_probe_parenthesized_receiver_freeze_bracket_source();

    assert_eq!(
        aliases,
        &[
            r#"Object.freeze((process)["kill"])(0)"#,
            r#"Object.freeze((process)["kill"])(+0)"#,
            r#"Object.freeze((globalThis.process)["kill"])(0)"#,
            r#"Object.freeze((globalThis.process)["kill"])(+0)"#,
            r#"Object.freeze((globalThis["process"])["kill"])(0)"#,
            r#"Object.freeze((globalThis["process"])["kill"])(+0)"#,
        ]
    );

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in parenthesized receiver-freeze bracket inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, format!("{};", aliases.join("; ")));
}

#[test]
fn test_process_kill_zero_probe_parenthesized_receiver_freeze_bracket_inventory_source_is_prefix_free_and_single_sourced(
) {
    let inventory_source =
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_inventory_source();
    let bracket_source = process_kill_zero_probe_parenthesized_receiver_freeze_bracket_source();
    let prefix_source = late_process_control_prefix_source();

    assert_eq!(inventory_source, bracket_source);
    assert_eq!(
        inventory_source.matches(bracket_source.trim_end()).count(),
        1
    );
    assert!(
        !inventory_source.contains(&prefix_source),
        "inventory: {inventory_source}"
    );
}

#[test]
fn test_process_kill_zero_probe_parenthesized_receiver_freeze_inventory_aliases_list_all_aliases_in_order(
) {
    let aliases = process_kill_zero_probe_parenthesized_receiver_freeze_inventory_aliases();
    let source = process_kill_zero_probe_parenthesized_receiver_freeze_inventory_source();

    assert_eq!(
        aliases,
        &[
            r#"Object.freeze((process)).kill(0)"#,
            r#"Object.freeze((process)).kill(+0)"#,
            r#"Object.freeze((globalThis.process)).kill(0)"#,
            r#"Object.freeze((globalThis.process)).kill(+0)"#,
            r#"Object.freeze((globalThis["process"])).kill(0)"#,
            r#"Object.freeze((globalThis["process"])).kill(+0)"#,
            r#"Object.freeze((process)["kill"])(0)"#,
            r#"Object.freeze((process)["kill"])(+0)"#,
            r#"Object.freeze((globalThis.process)["kill"])(0)"#,
            r#"Object.freeze((globalThis.process)["kill"])(+0)"#,
            r#"Object.freeze((globalThis["process"])["kill"])(0)"#,
            r#"Object.freeze((globalThis["process"])["kill"])(+0)"#,
        ]
    );

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in parenthesized receiver-freeze inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, format!("{};", aliases.join("; ")));
}

#[test]
fn test_process_kill_zero_probe_parenthesized_receiver_source_lists_all_aliases_in_order() {
    let aliases = process_kill_zero_probe_parenthesized_receiver_aliases();
    let source = process_kill_zero_probe_parenthesized_receiver_source();
    let expected = format!("{};", aliases.join("; "));

    assert_eq!(
        aliases,
        &[
            r#"((process)).kill(0)"#,
            r#"((process)).kill(+0)"#,
            r#"((globalThis.process)).kill(0)"#,
            r#"((globalThis.process)).kill(+0)"#,
        ]
    );
    assert_eq!(source, expected);
}
