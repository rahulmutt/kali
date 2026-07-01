use crate::*;

/// Canonical direct aliases for the supported Node `process.kill(0)` zero-probe slice.
pub const fn process_kill_zero_probe_direct_zero_aliases() -> &'static [&'static str] {
    &[
        r#"process.kill"#,
        r#"process["kill"]"#,
        r#"globalThis.process.kill"#,
        r#"globalThis.process["kill"]"#,
        r#"globalThis["process"].kill"#,
        r#"globalThis["process"]["kill"]"#,
        r#"process.kill(0)"#,
        r#"process.kill(+0)"#,
        r#"process["kill"](0)"#,
        r#"process["kill"](+0)"#,
        r#"globalThis.process.kill(0)"#,
        r#"globalThis.process.kill(+0)"#,
        r#"globalThis.process["kill"](0)"#,
        r#"globalThis.process["kill"](+0)"#,
        r#"globalThis["process"].kill(0)"#,
        r#"globalThis["process"].kill(+0)"#,
        r#"globalThis["process"]["kill"](0)"#,
        r#"globalThis["process"]["kill"](+0)"#,
    ]
}

/// Canonical wrapped-zero aliases for the supported Node `process.kill(0)` zero-probe slice.
pub const fn process_kill_zero_probe_wrapped_zero_aliases() -> &'static [&'static str] {
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
}

/// Canonical receiver-freeze dot aliases for the supported Node `process.kill(0)` zero-probe slice.
pub const fn process_kill_zero_probe_parenthesized_receiver_freeze_aliases(
) -> &'static [&'static str] {
    &[
        r#"Object.freeze((process)).kill(0)"#,
        r#"Object.freeze((process)).kill(+0)"#,
        r#"Object.freeze((globalThis.process)).kill(0)"#,
        r#"Object.freeze((globalThis.process)).kill(+0)"#,
        r#"Object.freeze((globalThis["process"])).kill(0)"#,
        r#"Object.freeze((globalThis["process"])).kill(+0)"#,
    ]
}

/// Canonical source text for the supported Node `process.kill(0)` receiver-freeze dot aliases.
pub fn process_kill_zero_probe_parenthesized_receiver_freeze_source() -> String {
    join_semicolon_terminated_segments(
        process_kill_zero_probe_parenthesized_receiver_freeze_aliases(),
    )
}

/// Canonical transparent parenthesized receiver aliases for the supported Node `process.kill(0)` slice.
pub const fn process_kill_zero_probe_parenthesized_receiver_aliases() -> &'static [&'static str] {
    &[
        r#"((process)).kill(0)"#,
        r#"((process)).kill(+0)"#,
        r#"((globalThis.process)).kill(0)"#,
        r#"((globalThis.process)).kill(+0)"#,
    ]
}

/// Canonical source text for the supported Node `process.kill(0)` transparent parenthesized receiver aliases.
pub fn process_kill_zero_probe_parenthesized_receiver_source() -> String {
    join_semicolon_terminated_segments(process_kill_zero_probe_parenthesized_receiver_aliases())
}

/// Canonical parenthesized frozen-callable aliases for the supported Node `process.kill(0)` slice.
pub const fn process_kill_zero_probe_parenthesized_frozen_callable_aliases(
) -> &'static [&'static str] {
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
}

/// Canonical source text for the supported Node `process.kill(0)` parenthesized frozen-callable aliases.
pub fn process_kill_zero_probe_parenthesized_frozen_callable_source() -> String {
    join_semicolon_terminated_segments(
        process_kill_zero_probe_parenthesized_frozen_callable_aliases(),
    )
}

/// Canonical parenthesized receiver-freeze bracket aliases for the supported Node `process.kill(0)` slice.
pub const fn process_kill_zero_probe_parenthesized_receiver_freeze_bracket_aliases(
) -> &'static [&'static str] {
    &[
        r#"Object.freeze((process)["kill"])(0)"#,
        r#"Object.freeze((process)["kill"])(+0)"#,
        r#"Object.freeze((globalThis.process)["kill"])(0)"#,
        r#"Object.freeze((globalThis.process)["kill"])(+0)"#,
        r#"Object.freeze((globalThis["process"])["kill"])(0)"#,
        r#"Object.freeze((globalThis["process"])["kill"])(+0)"#,
    ]
}

/// Canonical alias inventory for the supported Node `process.kill(0)` parenthesized receiver-freeze bracket aliases.
pub fn process_kill_zero_probe_parenthesized_receiver_freeze_bracket_inventory_aliases(
) -> Vec<&'static str> {
    ordered_unique_union(&[process_kill_zero_probe_parenthesized_receiver_freeze_bracket_aliases()])
}

/// Canonical source text for the supported Node `process.kill(0)` parenthesized receiver-freeze bracket aliases.
pub fn process_kill_zero_probe_parenthesized_receiver_freeze_bracket_inventory_source() -> String {
    join_semicolon_terminated_segments(
        &process_kill_zero_probe_parenthesized_receiver_freeze_bracket_inventory_aliases(),
    )
}

/// Canonical source text for the supported Node `process.kill(0)` parenthesized receiver-freeze bracket aliases.
pub fn process_kill_zero_probe_parenthesized_receiver_freeze_bracket_source() -> String {
    process_kill_zero_probe_parenthesized_receiver_freeze_bracket_inventory_source()
}

/// Canonical alias inventory for the supported Node `process.kill(0)` parenthesized receiver-freeze slice.
pub fn process_kill_zero_probe_parenthesized_receiver_freeze_inventory_aliases() -> Vec<&'static str>
{
    let bracket_aliases =
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_inventory_aliases();
    ordered_unique_union(&[
        process_kill_zero_probe_parenthesized_receiver_freeze_aliases(),
        bracket_aliases.as_slice(),
    ])
}

/// Canonical source text for the supported Node `process.kill(0)` parenthesized receiver-freeze aliases.
pub fn process_kill_zero_probe_parenthesized_receiver_freeze_inventory_source() -> String {
    join_semicolon_terminated_segments(
        &process_kill_zero_probe_parenthesized_receiver_freeze_inventory_aliases(),
    )
}

/// Canonical direct zero-probe source text for the supported Node `process.kill(0)` slice.
pub fn process_kill_zero_probe_direct_source() -> String {
    join_zero_probe_aliases(process_kill_zero_probe_direct_zero_aliases())
}

/// Canonical wrapped zero-probe source text for the supported Node `process.kill(0)` slice.
pub fn process_kill_zero_probe_wrapped_source() -> String {
    join_zero_probe_aliases(process_kill_zero_probe_wrapped_zero_aliases())
}

/// Canonical full alias inventory for the supported Node `process.kill(0)` zero-probe slice.
pub fn process_kill_zero_probe_aliases() -> Vec<&'static str> {
    ordered_unique_union(&[
        process_kill_zero_probe_direct_zero_aliases(),
        process_kill_zero_probe_wrapped_zero_aliases(),
    ])
}

/// Canonical call-target aliases for TS-wrapped supported Node `process.kill(0)` slices.
pub const fn process_kill_zero_probe_call_target_aliases() -> &'static [&'static str] {
    &[
        r#"process.kill"#,
        r#"globalThis.process.kill"#,
        r#"process["kill"]"#,
        r#"globalThis.process["kill"]"#,
        r#"globalThis["process"].kill"#,
        r#"globalThis["process"]["kill"]"#,
    ]
}

/// Canonical source text for the supported Node `process.kill(0)` TS-wrapped call-target aliases.
pub fn process_kill_zero_probe_call_target_inventory_source() -> String {
    join_semicolon_terminated_segments(process_kill_zero_probe_call_target_aliases())
}

/// Canonical source text for the supported Node `process.kill(0)` zero-probe call-target inventory wrapped in a typed expression.
pub fn process_kill_zero_probe_wrapped_call_target_source(argument_source: &str) -> String {
    let mut source = process_kill_zero_probe_call_target_aliases()
        .iter()
        .map(|alias| format!("{alias}{argument_source}"))
        .collect::<Vec<_>>()
        .join("; ");
    source.push(';');
    source
}

/// Canonical source text for the supported Node `process.kill(0)` zero-probe slices wrapped in a TS `satisfies` expression.
pub fn process_kill_zero_probe_satisfies_source() -> String {
    process_kill_zero_probe_wrapped_call_target_source("((0 satisfies number))")
}

/// Canonical source text for the supported Node `process.kill(0)` zero-probe slices wrapped in a TS type assertion.
pub fn process_kill_zero_probe_type_assertion_source() -> String {
    process_kill_zero_probe_wrapped_call_target_source("((0 as number))")
}

/// Canonical source text for the full supported Node `process.kill(0)` alias inventory.
///
/// This source composes the dedicated direct and wrapped zero-probe source helpers so the
/// inventory stays single-sourced.
pub fn process_kill_zero_probe_alias_inventory_source() -> String {
    format!(
        "{} {}",
        process_kill_zero_probe_direct_source().trim_end(),
        process_kill_zero_probe_wrapped_source().trim_end()
    )
}

/// Canonical zero-probe source text for the supported Node `process.kill(0)` slice.
pub fn process_kill_zero_probe_source() -> String {
    process_kill_zero_probe_alias_inventory_source()
}

/// Canonical binding inventory for the supported Node zero-probe sequence-callable-target bindings.
pub const fn process_kill_zero_probe_sequence_call_target_binding_lines(
) -> &'static [(&'static str, &'static str)] {
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
}

/// Canonical source text for the supported Node zero-probe sequence-callable-target bindings.
pub fn process_kill_zero_probe_sequence_call_target_bindings_source() -> String {
    join_const_binding_lines(process_kill_zero_probe_sequence_call_target_binding_lines())
}

/// Canonical binding inventory for the supported Node `process.kill(0)` direct call-target bindings.
pub const fn process_kill_zero_probe_call_target_binding_lines(
) -> &'static [(&'static str, &'static str)] {
    &[
        ("kill", "process.kill"),
        ("bracketedRootKill", "process[\"kill\"]"),
        ("dotRootKill", "globalThis.process.kill"),
        ("bracketedDotKill", "globalThis[\"process\"].kill"),
        ("dotBracketKill", "globalThis.process[\"kill\"]"),
        ("fullyBracketedKill", "globalThis[\"process\"][\"kill\"]"),
    ]
}

/// Canonical source text for the supported Node `process.kill(0)` direct call-target bindings.
pub fn process_kill_zero_probe_call_target_bindings_source() -> String {
    join_const_binding_lines(process_kill_zero_probe_call_target_binding_lines())
}

/// Canonical `console.log(...)` source text for the supported Node `process.kill(0)` slice.
pub fn process_kill_zero_probe_console_log_source() -> String {
    let statements = process_kill_zero_probe_aliases()
        .iter()
        .map(|alias| format!("console.log({alias})"))
        .collect::<Vec<_>>();
    format!("{};", statements.join("; "))
}

/// Canonical source text for the supported Node `process.kill(0)` node-API-surface
/// alias matrix used by the documented Node runtime regression.
pub fn process_kill_zero_probe_node_api_surface_run_source() -> String {
    format!(
        "const zero = 0; const zeroAlias = zero; {} {} {} {} console.log(process.kill(zeroAlias)); console.log(dotRootKill(+zero)); console.log(globalThis[\"process\"][\"kill\"](zero)); console.log(process[\"kill\"](zero)); console.log(kill(0)); console.log(bracketedDotKill(+0)); console.log(globalThis[\"process\"].kill(+0)); console.log(dotBracketKill(0)); console.log(fullyBracketedKill(0)); console.log(sequenceKill(0)); console.log(bracketedRootSequenceKill(0)); console.log(dotRootSequenceKill(0)); console.log(bracketedSequenceKill(0)); console.log(dotBracketSequenceKill(0)); console.log(bracketedDotSequenceKill(0)); console.log(globalThis[\"process\"][\"kill\"](+0)); console.log(((globalThis[\"process\"][\"kill\"]))(+0));\n",
        process_kill_zero_probe_call_target_bindings_source(),
        process_kill_zero_probe_sequence_call_target_bindings_source(),
        process_kill_zero_probe_parenthesized_receiver_freeze_source(),
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_source(),
    )
}

/// Canonical source text for the supported Node `process.kill(0)` node-API-surface
/// alias matrix used by the documented Node runtime regression.
pub fn process_kill_zero_probe_node_api_surface_test_source() -> String {
    format!(
        "const zero = 0; const zeroAlias = zero; {} {} {} {} globalThis[\"process\"].kill(+0); globalThis[\"process\"][\"kill\"](+0); Kali.test('process kill alias', () => {{ if ({}) {{ throw new Error('expected zero probe'); }} }});\n",
        process_kill_zero_probe_call_target_bindings_source(),
        process_kill_zero_probe_sequence_call_target_bindings_source(),
        process_kill_zero_probe_parenthesized_receiver_freeze_source(),
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_source(),
        process_kill_zero_probe_guard_source(),
    )
}

/// Canonical rejection-guard source text for the supported Node `process.kill(0)` slice.
pub fn process_kill_zero_probe_guard_source() -> String {
    process_kill_zero_probe_aliases()
        .iter()
        .map(|alias| format!("!{alias}"))
        .collect::<Vec<_>>()
        .join(" || ")
}

/// Canonical feature-unavailable wording for the supported Node `process.kill(0)` zero-probe slice.
pub fn process_kill_zero_probe_unavailable_message() -> String {
    let aliases = process_kill_zero_probe_aliases();
    format!(
        "process.kill is unavailable unless it is invoked as process.kill(0) or one of its supported Node zero-probe aliases: {}; use the zero liveness-probe subset or the later compatibility path",
        aliases.join(", ")
    )
}

#[cfg(test)]
#[path = "process_kill_tests.rs"]
mod process_kill_tests;
