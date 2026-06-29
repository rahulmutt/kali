use super::*;

#[test]
fn test_process_kill_zero_probe_source_lists_all_aliases_in_order() {
    let direct = process_kill_zero_probe_direct_zero_aliases();
    let wrapped = process_kill_zero_probe_wrapped_zero_aliases();
    let parenthesized_receiver_freeze_source =
        process_kill_zero_probe_parenthesized_receiver_freeze_source();
    let parenthesized_receiver_freeze_bracket_aliases =
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_aliases();
    let parenthesized_receiver_freeze_bracket_inventory_aliases =
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_inventory_aliases();
    let parenthesized_receiver_freeze_bracket_inventory_source =
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_inventory_source();
    let parenthesized_receiver_freeze_bracket_source =
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_source();
    let aliases = process_kill_zero_probe_aliases();
    let source = process_kill_zero_probe_source();
    let inventory_source = process_kill_zero_probe_alias_inventory_source();
    let direct_source = process_kill_zero_probe_direct_source();
    let wrapped_source = process_kill_zero_probe_wrapped_source();
    let expected = aliases.join("; ") + ";";

    for expected_alias in [
        r#"process.kill"#,
        r#"process.kill(0)"#,
        r#"process["kill"](0)"#,
        r#"process["kill"]((0))"#,
        r#"globalThis.process.kill"#,
        r#"globalThis.process["kill"](+0)"#,
        r#"globalThis["process"]["kill"]((0))"#,
        r#"process["kill"]((0))"#,
        r#"globalThis.process["kill"]((0))"#,
        r#"((globalThis.process.kill))(+0)"#,
        r#"globalThis["process"].kill((0))"#,
        r#"Object.freeze(process)["kill"](+0)"#,
        r#"Object.freeze((process)["kill"])(0)"#,
        r#"Object.freeze((process)["kill"])(+0)"#,
        r#"Object.freeze((process["kill"]))(0)"#,
        r#"Object.freeze((process["kill"]))(+0)"#,
        r#"Object.freeze(globalThis.process)["kill"](0)"#,
        r#"Object.freeze(globalThis.process)["kill"](+0)"#,
        r#"Object.freeze((globalThis.process)["kill"])(0)"#,
        r#"Object.freeze((globalThis.process)["kill"])(+0)"#,
        r#"Object.freeze((globalThis.process).kill)(0)"#,
        r#"Object.freeze((globalThis.process).kill)(+0)"#,
        r#"Object.freeze((globalThis["process"]).kill)(0)"#,
        r#"Object.freeze((globalThis["process"]).kill)(+0)"#,
        r#"Object.freeze((globalThis.process["kill"]))(0)"#,
        r#"Object.freeze((globalThis.process["kill"]))(+0)"#,
        r#"Object.freeze((process["kill"]))(0)"#,
        r#"Object.freeze((process["kill"]))(+0)"#,
        r#"Object.freeze((globalThis.process.kill))(0)"#,
        r#"Object.freeze((globalThis.process.kill))(+0)"#,
        r#"Object.freeze(globalThis["process"])["kill"](0)"#,
        r#"Object.freeze(globalThis["process"])["kill"](+0)"#,
        r#"Object.freeze((globalThis["process"])["kill"])(0)"#,
        r#"Object.freeze((globalThis["process"])["kill"])(+0)"#,
        r#"Object.freeze(globalThis.process["kill"])(0)"#,
        r#"Object.freeze(globalThis.process["kill"])(+0)"#,
        r#"Object.freeze(globalThis["process"].kill)(0)"#,
        r#"Object.freeze(globalThis["process"].kill)(+0)"#,
        r#"Object.freeze((globalThis["process"].kill))(0)"#,
        r#"Object.freeze((globalThis["process"].kill))(+0)"#,
        r#"Object.freeze(globalThis["process"]["kill"])(0)"#,
        r#"Object.freeze(globalThis["process"]["kill"])(+0)"#,
        r#"Object.freeze((globalThis["process"]["kill"]))(0)"#,
        r#"Object.freeze((globalThis["process"]["kill"]))(+0)"#,
        r#"((globalThis.process["kill"]))(0)"#,
        r#"((globalThis.process["kill"]))(+0)"#,
        r#"((process["kill"]))(0)"#,
        r#"((process["kill"]))(+0)"#,
        r#"((globalThis["process"]["kill"]))(0)"#,
        r#"((globalThis["process"]["kill"]))(+0)"#,
        r#"Object.freeze((process))["kill"](0)"#,
        r#"Object.freeze((process))["kill"](+0)"#,
        r#"Object.freeze((process)).kill(0)"#,
        r#"Object.freeze((process)).kill(+0)"#,
        r#"Object.freeze((globalThis.process))["kill"](0)"#,
        r#"Object.freeze((globalThis.process))["kill"](+0)"#,
        r#"Object.freeze((globalThis.process)).kill(0)"#,
        r#"Object.freeze((globalThis.process)).kill(+0)"#,
        r#"Object.freeze((globalThis["process"]))["kill"](0)"#,
        r#"Object.freeze((globalThis["process"]))["kill"](+0)"#,
        r#"Object.freeze((globalThis["process"])).kill(0)"#,
        r#"Object.freeze((globalThis["process"])).kill(+0)"#,
        r#"Object.freeze((process).kill)(0)"#,
        r#"Object.freeze((process).kill)(+0)"#,
    ] {
        assert!(
            aliases.contains(&expected_alias),
            "missing alias: {expected_alias}"
        );
    }

    let expected_aliases = direct
        .iter()
        .chain(wrapped.iter())
        .copied()
        .collect::<Vec<_>>();

    assert_eq!(aliases.len(), direct.len() + wrapped.len());
    assert_eq!(aliases, expected_aliases);
    assert!(direct.iter().all(|alias| !wrapped.contains(alias)));
    assert_eq!(direct_source, format!("{};", direct.join("; ")));
    assert_eq!(wrapped_source, format!("{};", wrapped.join("; ")));
    assert_eq!(inventory_source, expected);
    assert_eq!(
        inventory_source,
        format!("{} {}", direct_source.trim_end(), wrapped_source.trim_end())
    );
    assert_eq!(source, expected);
    assert_eq!(
        parenthesized_receiver_freeze_source,
        concat!(
            "Object.freeze((process)).kill(0); ",
            "Object.freeze((process)).kill(+0); ",
            "Object.freeze((globalThis.process)).kill(0); ",
            "Object.freeze((globalThis.process)).kill(+0); ",
            "Object.freeze((globalThis[\"process\"])).kill(0); ",
            "Object.freeze((globalThis[\"process\"])).kill(+0);"
        )
    );
    assert_eq!(
        parenthesized_receiver_freeze_bracket_aliases,
        &[
            r#"Object.freeze((process)["kill"])(0)"#,
            r#"Object.freeze((process)["kill"])(+0)"#,
            r#"Object.freeze((globalThis.process)["kill"])(0)"#,
            r#"Object.freeze((globalThis.process)["kill"])(+0)"#,
            r#"Object.freeze((globalThis["process"])["kill"])(0)"#,
            r#"Object.freeze((globalThis["process"])["kill"])(+0)"#,
        ]
    );
    assert_eq!(
        parenthesized_receiver_freeze_bracket_aliases,
        parenthesized_receiver_freeze_bracket_inventory_aliases
    );
    assert_eq!(
        parenthesized_receiver_freeze_bracket_source,
        concat!(
            "Object.freeze((process)[\"kill\"])(0); ",
            "Object.freeze((process)[\"kill\"])(+0); ",
            "Object.freeze((globalThis.process)[\"kill\"])(0); ",
            "Object.freeze((globalThis.process)[\"kill\"])(+0); ",
            "Object.freeze((globalThis[\"process\"])[\"kill\"])(0); ",
            "Object.freeze((globalThis[\"process\"])[\"kill\"])(+0);"
        )
    );
    assert_eq!(
        parenthesized_receiver_freeze_bracket_inventory_source,
        parenthesized_receiver_freeze_bracket_source
    );
}

#[test]
fn test_process_kill_zero_probe_alias_inventory_source_is_prefix_free_and_single_sourced() {
    let direct_source = process_kill_zero_probe_direct_source();
    let wrapped_source = process_kill_zero_probe_wrapped_source();
    let inventory_source = process_kill_zero_probe_alias_inventory_source();
    let prefix_source = late_process_control_prefix_source();

    assert_eq!(
        inventory_source,
        format!("{} {}", direct_source.trim_end(), wrapped_source.trim_end())
    );
    assert!(
        !inventory_source.contains(&prefix_source),
        "inventory: {inventory_source}"
    );
    assert!(
        !inventory_source.contains("process.kill(zeroAlias)"),
        "inventory: {inventory_source}"
    );
}

#[test]
fn test_process_kill_zero_probe_unavailable_message_lists_direct_and_wrapped_zero_aliases() {
    let aliases = process_kill_zero_probe_aliases();
    let message = process_kill_zero_probe_unavailable_message();
    let expected = format!(
        "process.kill is unavailable unless it is invoked as process.kill(0) or one of its supported Node zero-probe aliases: {}; use the zero liveness-probe subset or the later compatibility path",
        aliases.join(", ")
    );

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in zero-probe inventory: {alias}"
        );
    }

    assert_eq!(message, expected);
    assert_eq!(
        unique_aliases.len(),
        aliases.len(),
        "alias inventory should be duplicate-free"
    );
}

#[test]
fn test_process_kill_zero_probe_wrapped_zero_aliases_list_all_aliases_in_order() {
    let aliases = process_kill_zero_probe_wrapped_zero_aliases();
    let source = process_kill_zero_probe_wrapped_source();

    assert_eq!(
        aliases,
        &[
            r#"process.kill((0))"#,
            r#"process["kill"]((0))"#,
            r#"globalThis.process.kill((0))"#,
            r#"globalThis.process["kill"]((0))"#,
            r#"globalThis["process"].kill((0))"#,
            r#"globalThis["process"]["kill"]((0))"#,
            r#"Object.freeze(process.kill)(0)"#,
            r#"Object.freeze(process.kill)(+0)"#,
            r#"Object.freeze((process.kill))(0)"#,
            r#"Object.freeze((process.kill))(+0)"#,
            r#"Object.freeze(globalThis.process.kill)(0)"#,
            r#"Object.freeze(globalThis.process.kill)(+0)"#,
            r#"Object.freeze((globalThis.process.kill))(0)"#,
            r#"Object.freeze((globalThis.process.kill))(+0)"#,
            r#"Object.freeze(globalThis.process["kill"])(0)"#,
            r#"Object.freeze(globalThis.process["kill"])(+0)"#,
            r#"Object.freeze((globalThis.process["kill"]))(0)"#,
            r#"Object.freeze((globalThis.process["kill"]))(+0)"#,
            r#"Object.freeze(globalThis["process"].kill)(0)"#,
            r#"Object.freeze(globalThis["process"].kill)(+0)"#,
            r#"Object.freeze(globalThis["process"]["kill"])(0)"#,
            r#"Object.freeze(globalThis["process"]["kill"])(+0)"#,
            r#"Object.freeze((globalThis["process"]["kill"]))(0)"#,
            r#"Object.freeze((globalThis["process"]["kill"]))(+0)"#,
            r#"Object.freeze((globalThis["process"].kill))(0)"#,
            r#"Object.freeze((globalThis["process"].kill))(+0)"#,
            r#"Object.freeze(process)["kill"](0)"#,
            r#"Object.freeze(process)["kill"](+0)"#,
            r#"Object.freeze((process)["kill"])(0)"#,
            r#"Object.freeze((process)["kill"])(+0)"#,
            r#"Object.freeze((process).kill)(0)"#,
            r#"Object.freeze((process).kill)(+0)"#,
            r#"Object.freeze((process["kill"]))(0)"#,
            r#"Object.freeze((process["kill"]))(+0)"#,
            r#"Object.freeze(globalThis.process)["kill"](0)"#,
            r#"Object.freeze(globalThis.process)["kill"](+0)"#,
            r#"Object.freeze((globalThis.process)["kill"])(0)"#,
            r#"Object.freeze((globalThis.process)["kill"])(+0)"#,
            r#"Object.freeze((globalThis.process).kill)(0)"#,
            r#"Object.freeze((globalThis.process).kill)(+0)"#,
            r#"Object.freeze((globalThis["process"]).kill)(0)"#,
            r#"Object.freeze((globalThis["process"]).kill)(+0)"#,
            r#"Object.freeze(globalThis["process"])["kill"](0)"#,
            r#"Object.freeze(globalThis["process"])["kill"](+0)"#,
            r#"Object.freeze((globalThis["process"])["kill"])(0)"#,
            r#"Object.freeze((globalThis["process"])["kill"])(+0)"#,
            r#"((process.kill))(0)"#,
            r#"((process.kill))(+0)"#,
            r#"((globalThis.process.kill))(0)"#,
            r#"((globalThis.process.kill))(+0)"#,
            r#"((process["kill"]))(0)"#,
            r#"((process["kill"]))(+0)"#,
            r#"((globalThis.process["kill"]))(0)"#,
            r#"((globalThis.process["kill"]))(+0)"#,
            r#"((globalThis["process"].kill))(0)"#,
            r#"((globalThis["process"].kill))(+0)"#,
            r#"((globalThis["process"]["kill"]))(0)"#,
            r#"((globalThis["process"]["kill"]))(+0)"#,
            r#"Object.freeze((process))["kill"](0)"#,
            r#"Object.freeze((process))["kill"](+0)"#,
            r#"Object.freeze((process)).kill(0)"#,
            r#"Object.freeze((process)).kill(+0)"#,
            r#"Object.freeze((globalThis.process))["kill"](0)"#,
            r#"Object.freeze((globalThis.process))["kill"](+0)"#,
            r#"Object.freeze((globalThis.process)).kill(0)"#,
            r#"Object.freeze((globalThis.process)).kill(+0)"#,
            r#"Object.freeze((globalThis["process"]))["kill"](0)"#,
            r#"Object.freeze((globalThis["process"]))["kill"](+0)"#,
            r#"Object.freeze((globalThis["process"])).kill(0)"#,
            r#"Object.freeze((globalThis["process"])).kill(+0)"#,
        ]
    );

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in wrapped zero-probe inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, format!("{};", aliases.join("; ")));
}

#[test]
fn test_process_kill_zero_probe_console_log_source_lists_all_aliases_in_order() {
    let aliases = process_kill_zero_probe_aliases();
    let source = process_kill_zero_probe_console_log_source();
    let expected = aliases
        .iter()
        .map(|alias| format!("console.log({alias})"))
        .collect::<Vec<_>>()
        .join("; ");

    assert_eq!(source, format!("{expected};"));
}

#[test]
fn test_process_kill_zero_probe_guard_source_lists_all_aliases_in_order() {
    let aliases = process_kill_zero_probe_aliases();
    let source = process_kill_zero_probe_guard_source();
    let expected = aliases
        .iter()
        .map(|alias| format!("!{alias}"))
        .collect::<Vec<_>>()
        .join(" || ");

    assert_eq!(source, expected);
}
