use super::*;

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
