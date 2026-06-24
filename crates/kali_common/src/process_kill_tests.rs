use crate::*;

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
fn test_process_kill_zero_probe_node_api_surface_sources_are_canonical() {
    let run_source = process_kill_zero_probe_node_api_surface_run_source();
    let test_source = process_kill_zero_probe_node_api_surface_test_source();
    let expected_run = format!(
        "const zero = 0; const zeroAlias = zero; {} {} {} {} console.log(process.kill(zeroAlias)); console.log(dotRootKill(+zero)); console.log(globalThis[\"process\"][\"kill\"](zero)); console.log(process[\"kill\"](zero)); console.log(kill(0)); console.log(bracketedDotKill(+0)); console.log(globalThis[\"process\"].kill(+0)); console.log(dotBracketKill(0)); console.log(fullyBracketedKill(0)); console.log(sequenceKill(0)); console.log(bracketedRootSequenceKill(0)); console.log(dotRootSequenceKill(0)); console.log(bracketedSequenceKill(0)); console.log(dotBracketSequenceKill(0)); console.log(bracketedDotSequenceKill(0)); console.log(globalThis[\"process\"][\"kill\"](+0)); console.log(((globalThis[\"process\"][\"kill\"]))(+0));\n",
        process_kill_zero_probe_call_target_bindings_source(),
        process_kill_zero_probe_sequence_call_target_bindings_source(),
        process_kill_zero_probe_parenthesized_receiver_freeze_source(),
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_source(),
    );
    let expected_test = format!(
        "const zero = 0; const zeroAlias = zero; {} {} {} {} globalThis[\"process\"].kill(+0); globalThis[\"process\"][\"kill\"](+0); Kali.test('process kill alias', () => {{ if ({}) {{ throw new Error('expected zero probe'); }} }});\n",
        process_kill_zero_probe_call_target_bindings_source(),
        process_kill_zero_probe_sequence_call_target_bindings_source(),
        process_kill_zero_probe_parenthesized_receiver_freeze_source(),
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_source(),
        process_kill_zero_probe_guard_source(),
    );

    assert_eq!(run_source, expected_run);
    assert_eq!(test_source, expected_test);
}

#[test]
fn test_process_kill_zero_probe_call_target_aliases_list_all_supported_targets_in_order() {
    let targets = process_kill_zero_probe_call_target_aliases();

    assert_eq!(
        targets,
        &[
            r#"process.kill"#,
            r#"globalThis.process.kill"#,
            r#"process["kill"]"#,
            r#"globalThis.process["kill"]"#,
            r#"globalThis["process"].kill"#,
            r#"globalThis["process"]["kill"]"#,
        ]
    );

    let mut unique_targets = std::collections::HashSet::new();
    for target in targets.iter().copied() {
        assert!(
            unique_targets.insert(target),
            "duplicate call-target alias in typed zero-probe inventory: {target}"
        );
    }

    assert_eq!(targets.len(), unique_targets.len());
}

#[test]
fn test_process_kill_zero_probe_typed_wrapper_sources_list_all_call_targets_in_order() {
    let targets = process_kill_zero_probe_call_target_aliases();
    let inventory_source = process_kill_zero_probe_call_target_inventory_source();
    let satisfies_source = process_kill_zero_probe_satisfies_source();
    let type_assertion_source = process_kill_zero_probe_type_assertion_source();
    let expected_inventory = format!("{};", targets.join("; "));
    let expected_satisfies = format!(
        "{};",
        targets
            .iter()
            .map(|alias| format!("{alias}((0 satisfies number))"))
            .collect::<Vec<_>>()
            .join("; ")
    );
    let expected_type_assertion = format!(
        "{};",
        targets
            .iter()
            .map(|alias| format!("{alias}((0 as number))"))
            .collect::<Vec<_>>()
            .join("; ")
    );

    let mut unique_targets = std::collections::HashSet::new();
    for target in targets.iter().copied() {
        assert!(
            unique_targets.insert(target),
            "duplicate call-target alias in typed zero-probe inventory: {target}"
        );
    }

    assert_eq!(targets.len(), unique_targets.len());
    assert_eq!(inventory_source, expected_inventory);
    assert_eq!(satisfies_source, expected_satisfies);
    assert_eq!(type_assertion_source, expected_type_assertion);
}

#[test]
fn test_process_kill_zero_probe_wrapped_call_target_source_reuses_the_shared_inventory() {
    let targets = process_kill_zero_probe_call_target_aliases();
    let expected_satisfies = format!(
        "{};",
        targets
            .iter()
            .map(|alias| format!("{alias}((0 satisfies number))"))
            .collect::<Vec<_>>()
            .join("; ")
    );
    let expected_type_assertion = format!(
        "{};",
        targets
            .iter()
            .map(|alias| format!("{alias}((0 as number))"))
            .collect::<Vec<_>>()
            .join("; ")
    );

    assert_eq!(
        process_kill_zero_probe_wrapped_call_target_source("((0 satisfies number))"),
        expected_satisfies
    );
    assert_eq!(
        process_kill_zero_probe_wrapped_call_target_source("((0 as number))"),
        expected_type_assertion
    );
}

#[test]
fn test_process_kill_zero_probe_call_target_aliases_are_in_canonical_order() {
    assert_eq!(
        process_kill_zero_probe_call_target_aliases(),
        &[
            r#"process.kill"#,
            r#"globalThis.process.kill"#,
            r#"process["kill"]"#,
            r#"globalThis.process["kill"]"#,
            r#"globalThis["process"].kill"#,
            r#"globalThis["process"]["kill"]"#,
        ]
    );
    assert_eq!(
        process_kill_zero_probe_call_target_inventory_source(),
        concat!(
            r#"process.kill; "#,
            r#"globalThis.process.kill; "#,
            r#"process["kill"]; "#,
            r#"globalThis.process["kill"]; "#,
            r#"globalThis["process"].kill; "#,
            r#"globalThis["process"]["kill"];"#
        )
    );
}

#[test]
fn test_process_kill_zero_probe_direct_call_target_binding_lines_are_canonical() {
    assert_eq!(
        process_kill_zero_probe_call_target_binding_lines(),
        &[
            ("kill", "process.kill"),
            ("bracketedRootKill", "process[\"kill\"]"),
            ("dotRootKill", "globalThis.process.kill"),
            ("bracketedDotKill", "globalThis[\"process\"].kill"),
            ("dotBracketKill", "globalThis.process[\"kill\"]"),
            ("fullyBracketedKill", "globalThis[\"process\"][\"kill\"]"),
        ]
    );
    assert_eq!(
        process_kill_zero_probe_call_target_bindings_source(),
        concat!(
            "const kill = process.kill; ",
            "const bracketedRootKill = process[\"kill\"]; ",
            "const dotRootKill = globalThis.process.kill; ",
            "const bracketedDotKill = globalThis[\"process\"].kill; ",
            "const dotBracketKill = globalThis.process[\"kill\"]; ",
            "const fullyBracketedKill = globalThis[\"process\"][\"kill\"];"
        )
    );
}

#[test]
fn test_process_kill_zero_probe_sequence_call_target_binding_lines_are_canonical() {
    assert_eq!(
        process_kill_zero_probe_sequence_call_target_binding_lines(),
        &[
            ("sequenceKill", "(process.kill, process.kill)"),
            (
                "bracketedRootSequenceKill",
                "(process[\"kill\"], process[\"kill\"])",
            ),
            (
                "dotRootSequenceKill",
                "(globalThis.process.kill, globalThis.process.kill)",
            ),
            (
                "bracketedSequenceKill",
                "(globalThis[\"process\"][\"kill\"], globalThis[\"process\"][\"kill\"])",
            ),
            (
                "dotBracketSequenceKill",
                "(globalThis.process[\"kill\"], globalThis.process[\"kill\"])",
            ),
            (
                "bracketedDotSequenceKill",
                "(globalThis[\"process\"].kill, globalThis[\"process\"].kill)",
            ),
        ]
    );
    assert_eq!(
        process_kill_zero_probe_sequence_call_target_bindings_source(),
        concat!(
            "const sequenceKill = (process.kill, process.kill); ",
            "const bracketedRootSequenceKill = (process[\"kill\"], process[\"kill\"]); ",
            "const dotRootSequenceKill = (globalThis.process.kill, globalThis.process.kill); ",
            "const bracketedSequenceKill = (globalThis[\"process\"][\"kill\"], globalThis[\"process\"][\"kill\"]); ",
            "const dotBracketSequenceKill = (globalThis.process[\"kill\"], globalThis.process[\"kill\"]); ",
            "const bracketedDotSequenceKill = (globalThis[\"process\"].kill, globalThis[\"process\"].kill);"
        )
    );
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
