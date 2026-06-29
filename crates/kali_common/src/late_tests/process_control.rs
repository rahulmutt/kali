use super::*;

#[test]
fn test_late_process_control_prefix_source_lists_all_prefix_aliases_in_order() {
    let prefix = late_process_control_prefix_source();
    let expected = format!(
        "{}; {}",
        format!("{};", LATE_PROCESS_CONTROL_PREFIX_SEGMENTS.join("; ")).trim_end_matches(';'),
        late_process_control_exit_source().trim_end()
    );

    assert_eq!(prefix, expected);
    assert!(
        prefix.starts_with("Deno.pid; globalThis.Deno.pid;"),
        "prefix: {prefix}"
    );
    assert!(
        prefix.contains("process.kill; globalThis.process.kill;"),
        "prefix: {prefix}"
    );
    assert!(
        prefix.contains(r#"globalThis["process"].exit"#),
        "prefix: {prefix}"
    );
    assert!(
        !prefix.contains("Object.freeze((process)).kill(0)"),
        "prefix: {prefix}"
    );
    assert!(!prefix.contains("process.kill(0)"), "prefix: {prefix}");
}

#[test]
fn test_late_process_control_exit_aliases_are_canonical() {
    assert_eq!(
        late_process_control_exit_aliases(),
        &[
            "process.exit",
            "globalThis.process.exit",
            "globalThis.process[\"exit\"]",
            r#"globalThis["process"].exit"#,
            r#"globalThis["process"]["exit"]"#,
            "process[\"exit\"]",
        ]
    );
}

#[test]
fn test_late_process_control_exit_source_lists_all_aliases_in_order() {
    let source = late_process_control_exit_source();
    let expected = concat!(
        "process.exit; ",
        "globalThis.process.exit; ",
        "globalThis.process[\"exit\"]; ",
        "globalThis[\"process\"].exit; ",
        "globalThis[\"process\"][\"exit\"]; ",
        "process[\"exit\"];"
    );

    assert_eq!(source, expected);
}

#[test]
fn test_late_process_control_source_reuses_the_shared_zero_probe_inventory_once() {
    let source = late_process_control_source();
    let prefix = late_process_control_prefix_source();
    let zero_probe_source = process_kill_zero_probe_source();

    assert!(source.starts_with(&prefix), "source: {source}");
    assert!(source.contains(&zero_probe_source), "source: {source}");
    assert_eq!(
        source.matches(&zero_probe_source).count(),
        1,
        "late process control source should embed the zero-probe inventory exactly once"
    );
    let parenthesized_receiver_aliases = process_kill_zero_probe_parenthesized_receiver_aliases();
    assert_eq!(
        parenthesized_receiver_aliases,
        &[
            r#"((process)).kill(0)"#,
            r#"((process)).kill(+0)"#,
            r#"((globalThis.process)).kill(0)"#,
            r#"((globalThis.process)).kill(+0)"#,
        ]
    );
    let parenthesized_receiver_source = process_kill_zero_probe_parenthesized_receiver_source();
    assert!(
        source.contains(parenthesized_receiver_source.trim_end()),
        "source: {source}"
    );
    assert_eq!(
        source.matches(parenthesized_receiver_source.trim_end()).count(),
        1,
        "late process control source should embed the transparent parenthesized receiver aliases exactly once"
    );
    assert!(
        source.contains("process.kill(zeroAlias)"),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process).kill)(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process).kill)(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis["process"]["kill"]))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis["process"]["kill"]))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis["process"])["kill"])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis["process"])["kill"])(+0)"#),
        "source: {source}"
    );
    let parenthesized_receiver_freeze_aliases =
        process_kill_zero_probe_parenthesized_receiver_freeze_aliases();
    assert_eq!(
        parenthesized_receiver_freeze_aliases,
        &[
            r#"Object.freeze((process)).kill(0)"#,
            r#"Object.freeze((process)).kill(+0)"#,
            r#"Object.freeze((globalThis.process)).kill(0)"#,
            r#"Object.freeze((globalThis.process)).kill(+0)"#,
            r#"Object.freeze((globalThis["process"])).kill(0)"#,
            r#"Object.freeze((globalThis["process"])).kill(+0)"#,
        ]
    );
    let parenthesized_receiver_freeze_source =
        process_kill_zero_probe_parenthesized_receiver_freeze_source();
    assert!(
        source.contains(parenthesized_receiver_freeze_source.trim_end()),
        "source: {source}"
    );
    assert_eq!(
        source
            .matches(parenthesized_receiver_freeze_source.trim_end())
            .count(),
        1,
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process)).kill(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process)).kill(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis.process)).kill(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis.process)).kill(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis["process"])).kill(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis["process"])).kill(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"((process)).kill(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"((process)).kill(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"((globalThis.process)).kill(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"((process["kill"]))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"((process["kill"]))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"((globalThis["process"])).kill(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"((globalThis["process"])).kill(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"]["cwd"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"]["chdir"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"].exit"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"]["exit"]"#),
        "source: {source}"
    );
    assert!(
        prefix.ends_with("process[\"exit\"];"),
        "prefix should preserve the process-control preamble: {prefix}"
    );
}

#[test]
fn test_late_process_control_single_quoted_process_source_reuses_the_shared_zero_probe_inventory_once(
) {
    let source = late_process_control_single_quoted_process_source();
    let zero_probe_source = late_process_control_source();
    let single_quoted_process_source =
        join_semicolon_terminated_segments(late_process_control_single_quoted_process_aliases());
    let expected = format!(
        "{} {}",
        zero_probe_source,
        single_quoted_process_source.trim_end()
    );

    assert_eq!(source, expected, "source: {source}");
    for expected in [
        r#"; process['kill']((0));"#,
        r#"; globalThis['process'].kill((0));"#,
        r#"; process['exit']((0));"#,
        r#"; globalThis['process'].exit((0));"#,
        r#"; Object.freeze((process)['exit'])(0);"#,
        r#"; Object.freeze((process)['exit'])(+0);"#,
        r#"; Object.freeze((globalThis.process)['exit'])(0);"#,
        r#"; Object.freeze((globalThis.process)['exit'])(+0);"#,
        r#"; Object.freeze((globalThis['process'])['exit'])(0);"#,
        r#"; Object.freeze((globalThis['process'])['exit'])(+0);"#,
    ] {
        assert_eq!(
            source.matches(expected).count(),
            1,
            "wrapped single-quoted alias should appear exactly once: {expected}; source: {source}"
        );
    }
    assert!(
        source.contains(r#"globalThis['process'].kill(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process'].kill(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process']['kill'](0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process']['kill'](+0)"#),
        "source: {source}"
    );
    assert!(source.contains(r#"process['kill'](0)"#), "source: {source}");
    assert!(
        source.contains(r#"process['kill'](+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"process['kill']((0))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.process['kill'](0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.process['kill'](+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.process['kill']((0))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process'].kill((0))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process']['kill']((0))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.process['kill'](+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(process['kill'])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(process['kill'])(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process['kill']))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process['kill']))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis.process['kill'])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis.process['kill'])(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis.process['kill']))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis.process['kill']))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process'].kill)(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process'].kill)(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process']).kill)(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process']).kill)(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process'])['kill'])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process'])['kill'])(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process'].kill))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process'].kill))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process']['kill']))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process']['kill']))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process']['kill'])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process']['kill'])(+0)"#),
        "source: {source}"
    );
    assert!(source.contains(r#"process['exit'](0)"#), "source: {source}");
    assert!(
        source.contains(r#"process['exit'](+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(process['exit'])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(process['exit'])(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process['exit']))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((process['exit']))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process'].exit(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process'].exit(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process'].exit((0))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process']['exit'](0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process']['exit'](+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process']['exit']((0))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.process['exit']((0))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis['process']['exit'](+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process'].exit)(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process'].exit)(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process'].exit))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process'].exit))(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process']['exit'])(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['process']['exit'])(+0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process']['exit']))(0)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['process']['exit']))(+0)"#),
        "source: {source}"
    );
}

#[test]
fn test_late_process_control_single_quoted_process_aliases_lists_all_aliases_in_order() {
    let aliases = late_process_control_single_quoted_process_aliases();
    let aliases_source = late_process_control_single_quoted_process_aliases_source();
    let source = late_process_control_single_quoted_process_source();
    let expected_segment = aliases.join("; ");

    assert_eq!(
        aliases,
        &[
            r#"globalThis['process'].kill(0)"#,
            r#"globalThis['process'].kill(+0)"#,
            r#"globalThis['process']['kill'](0)"#,
            r#"globalThis['process']['kill'](+0)"#,
            r#"process['kill'](0)"#,
            r#"process['kill'](+0)"#,
            r#"process['kill']((0))"#,
            r#"globalThis.process['kill'](0)"#,
            r#"globalThis.process['kill'](+0)"#,
            r#"globalThis.process['kill']((0))"#,
            r#"globalThis['process'].kill((0))"#,
            r#"globalThis['process']['kill']((0))"#,
            r#"globalThis.process['kill']((0))"#,
            r#"Object.freeze(process['kill'])(0)"#,
            r#"Object.freeze(process['kill'])(+0)"#,
            r#"Object.freeze((process['kill']))(0)"#,
            r#"Object.freeze((process['kill']))(+0)"#,
            r#"Object.freeze(globalThis.process['kill'])(0)"#,
            r#"Object.freeze(globalThis.process['kill'])(+0)"#,
            r#"Object.freeze((globalThis.process['kill']))(0)"#,
            r#"Object.freeze((globalThis.process['kill']))(+0)"#,
            r#"Object.freeze(globalThis['process'].kill)(0)"#,
            r#"Object.freeze(globalThis['process'].kill)(+0)"#,
            r#"Object.freeze((globalThis['process']).kill)(0)"#,
            r#"Object.freeze((globalThis['process']).kill)(+0)"#,
            r#"Object.freeze((globalThis['process'])['kill'])(0)"#,
            r#"Object.freeze((globalThis['process'])['kill'])(+0)"#,
            r#"Object.freeze((globalThis['process'].kill))(0)"#,
            r#"Object.freeze((globalThis['process'].kill))(+0)"#,
            r#"Object.freeze((globalThis['process']['kill']))(0)"#,
            r#"Object.freeze((globalThis['process']['kill']))(+0)"#,
            r#"Object.freeze(globalThis['process']['kill'])(0)"#,
            r#"Object.freeze(globalThis['process']['kill'])(+0)"#,
            r#"process['exit'](0)"#,
            r#"process['exit'](+0)"#,
            r#"process['exit']((0))"#,
            r#"Object.freeze(process['exit'])(0)"#,
            r#"Object.freeze(process['exit'])(+0)"#,
            r#"Object.freeze((process['exit']))(0)"#,
            r#"Object.freeze((process['exit']))(+0)"#,
            r#"Object.freeze((process)['exit'])(0)"#,
            r#"Object.freeze((process)['exit'])(+0)"#,
            r#"Object.freeze((globalThis.process)['exit'])(0)"#,
            r#"Object.freeze((globalThis.process)['exit'])(+0)"#,
            r#"Object.freeze((globalThis['process'])['exit'])(0)"#,
            r#"Object.freeze((globalThis['process'])['exit'])(+0)"#,
            r#"globalThis['process'].exit(0)"#,
            r#"globalThis['process'].exit(+0)"#,
            r#"globalThis['process'].exit((0))"#,
            r#"globalThis['process']['exit'](0)"#,
            r#"globalThis['process']['exit'](+0)"#,
            r#"globalThis['process']['exit']((0))"#,
            r#"globalThis.process['exit'](0)"#,
            r#"globalThis.process['exit'](+0)"#,
            r#"globalThis.process['exit']((0))"#,
            r#"Object.freeze(globalThis['process'].exit)(0)"#,
            r#"Object.freeze(globalThis['process'].exit)(+0)"#,
            r#"Object.freeze((globalThis['process'].exit))(0)"#,
            r#"Object.freeze((globalThis['process'].exit))(+0)"#,
            r#"Object.freeze(globalThis['process']['exit'])(0)"#,
            r#"Object.freeze(globalThis['process']['exit'])(+0)"#,
            r#"Object.freeze((globalThis['process']['exit']))(0)"#,
            r#"Object.freeze((globalThis['process']['exit']))(+0)"#,
        ]
    );
    assert_eq!(aliases_source, format!("{};", expected_segment));
    assert_eq!(
        source.matches(&expected_segment).count(),
        1,
        "source: {source}"
    );
}

#[test]
fn test_late_process_control_single_quoted_process_aliases_compose_kill_and_exit_helpers() {
    let aliases = late_process_control_single_quoted_process_aliases();
    let kill_aliases = late_process_control_single_quoted_kill_aliases();
    let exit_aliases = late_process_control_single_quoted_exit_aliases();
    let kill_source = late_process_control_single_quoted_kill_source();
    let exit_source = late_process_control_single_quoted_exit_source();
    let source = late_process_control_single_quoted_process_aliases_source();

    assert_eq!(kill_aliases, &aliases[..kill_aliases.len()]);
    assert_eq!(exit_aliases, &aliases[kill_aliases.len()..]);
    assert_eq!(
        source,
        format!("{} {}", kill_source.trim_end(), exit_source.trim_end())
    );
    assert_eq!(
        source.matches(kill_source.trim_end()).count(),
        1,
        "source: {source}"
    );
    assert_eq!(
        source.matches(exit_source.trim_end()).count(),
        1,
        "source: {source}"
    );
}

#[test]
fn test_late_process_control_single_quoted_kill_source_lists_all_aliases_in_order() {
    let aliases = late_process_control_single_quoted_kill_aliases();
    let source = late_process_control_single_quoted_kill_source();
    let expected = format!("{};", aliases.join("; "));

    assert_eq!(
        aliases,
        &[
            r#"globalThis['process'].kill(0)"#,
            r#"globalThis['process'].kill(+0)"#,
            r#"globalThis['process']['kill'](0)"#,
            r#"globalThis['process']['kill'](+0)"#,
            r#"process['kill'](0)"#,
            r#"process['kill'](+0)"#,
            r#"process['kill']((0))"#,
            r#"globalThis.process['kill'](0)"#,
            r#"globalThis.process['kill'](+0)"#,
            r#"globalThis.process['kill']((0))"#,
            r#"globalThis['process'].kill((0))"#,
            r#"globalThis['process']['kill']((0))"#,
            r#"globalThis.process['kill']((0))"#,
            r#"Object.freeze(process['kill'])(0)"#,
            r#"Object.freeze(process['kill'])(+0)"#,
            r#"Object.freeze((process['kill']))(0)"#,
            r#"Object.freeze((process['kill']))(+0)"#,
            r#"Object.freeze(globalThis.process['kill'])(0)"#,
            r#"Object.freeze(globalThis.process['kill'])(+0)"#,
            r#"Object.freeze((globalThis.process['kill']))(0)"#,
            r#"Object.freeze((globalThis.process['kill']))(+0)"#,
            r#"Object.freeze(globalThis['process'].kill)(0)"#,
            r#"Object.freeze(globalThis['process'].kill)(+0)"#,
            r#"Object.freeze((globalThis['process']).kill)(0)"#,
            r#"Object.freeze((globalThis['process']).kill)(+0)"#,
            r#"Object.freeze((globalThis['process'])['kill'])(0)"#,
            r#"Object.freeze((globalThis['process'])['kill'])(+0)"#,
            r#"Object.freeze((globalThis['process'].kill))(0)"#,
            r#"Object.freeze((globalThis['process'].kill))(+0)"#,
            r#"Object.freeze((globalThis['process']['kill']))(0)"#,
            r#"Object.freeze((globalThis['process']['kill']))(+0)"#,
            r#"Object.freeze(globalThis['process']['kill'])(0)"#,
            r#"Object.freeze(globalThis['process']['kill'])(+0)"#,
        ]
    );
    assert_eq!(source, expected, "source: {source}");
}

#[test]
fn test_late_process_control_single_quoted_exit_source_lists_all_aliases_in_order() {
    let aliases = late_process_control_single_quoted_exit_aliases();
    let source = late_process_control_single_quoted_exit_aliases_source();
    let expected = format!("{};", aliases.join("; "));

    assert_eq!(
        aliases,
        &[
            r#"process['exit'](0)"#,
            r#"process['exit'](+0)"#,
            r#"process['exit']((0))"#,
            r#"Object.freeze(process['exit'])(0)"#,
            r#"Object.freeze(process['exit'])(+0)"#,
            r#"Object.freeze((process['exit']))(0)"#,
            r#"Object.freeze((process['exit']))(+0)"#,
            r#"Object.freeze((process)['exit'])(0)"#,
            r#"Object.freeze((process)['exit'])(+0)"#,
            r#"Object.freeze((globalThis.process)['exit'])(0)"#,
            r#"Object.freeze((globalThis.process)['exit'])(+0)"#,
            r#"Object.freeze((globalThis['process'])['exit'])(0)"#,
            r#"Object.freeze((globalThis['process'])['exit'])(+0)"#,
            r#"globalThis['process'].exit(0)"#,
            r#"globalThis['process'].exit(+0)"#,
            r#"globalThis['process'].exit((0))"#,
            r#"globalThis['process']['exit'](0)"#,
            r#"globalThis['process']['exit'](+0)"#,
            r#"globalThis['process']['exit']((0))"#,
            r#"globalThis.process['exit'](0)"#,
            r#"globalThis.process['exit'](+0)"#,
            r#"globalThis.process['exit']((0))"#,
            r#"Object.freeze(globalThis['process'].exit)(0)"#,
            r#"Object.freeze(globalThis['process'].exit)(+0)"#,
            r#"Object.freeze((globalThis['process'].exit))(0)"#,
            r#"Object.freeze((globalThis['process'].exit))(+0)"#,
            r#"Object.freeze(globalThis['process']['exit'])(0)"#,
            r#"Object.freeze(globalThis['process']['exit'])(+0)"#,
            r#"Object.freeze((globalThis['process']['exit']))(0)"#,
            r#"Object.freeze((globalThis['process']['exit']))(+0)"#,
        ]
    );
    assert_eq!(source, expected);
    assert_eq!(source.matches(&expected).count(), 1, "source: {source}");
    assert!(!source.contains("process.kill(0)"), "source: {source}");
}

#[test]
fn test_late_process_env_mutation_source_lists_mixed_quote_process_aliases_and_mixed_delete_aliases(
) {
    let aliases = late_process_env_mutation_aliases();
    let source = late_process_env_mutation_source();
    let expected = format!("{};", aliases.join("; "));

    assert_eq!(source, expected);
    for fragment in [
        r#"process['env'] = {}"#,
        r#"process['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
        r#"globalThis.process['env'] = {}"#,
        r#"globalThis.process['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
        r#"globalThis["process"]['env'] = {}"#,
        r#"globalThis["process"]['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
        r#"globalThis['process']["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
        r#"globalThis['process']["env"] = {}"#,
        r#"globalThis['process']["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
        r#"delete globalThis.process['env']['KALI_BROWSER_ENV_MUTATION']"#,
        r#"delete globalThis["process"]['env']['KALI_BROWSER_ENV_MUTATION']"#,
        r#"delete globalThis['process']["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
        r#"delete globalThis['process']['env']['KALI_BROWSER_ENV_MUTATION']"#,
    ] {
        assert!(
            source.contains(fragment),
            "missing fragment: {fragment}; source: {source}"
        );
    }
}
