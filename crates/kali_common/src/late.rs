use crate::*;

/// Canonical source text for the supported late-compat `Object.hasOwn` / `Object.prototype.hasOwnProperty.call` slice.
pub fn late_compat_object_has_own_source(receiver_source: &str, key_source: &str) -> String {
    let source = [
        format!("Object.hasOwn({receiver_source}, {key_source})"),
        format!("globalThis.Object.hasOwn({receiver_source}, {key_source})"),
        format!(r#"globalThis.Object["hasOwn"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"].hasOwn({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"]["hasOwn"]({receiver_source}, {key_source})"#),
        format!(r#"Object["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"Object["hasOwnProperty"]["call"]({receiver_source}, {key_source})"#),
        format!(r#"Object.prototype.hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"Object.prototype.hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"Object["prototype"].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"Object["prototype"].hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"Object["prototype"]["hasOwnProperty"]["call"]({receiver_source}, {key_source})"#),
        format!(r#"Object.prototype["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object['hasOwnProperty'].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"]['hasOwnProperty'].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"]["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object']['hasOwnProperty'].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].prototype.hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].prototype.hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].prototype['hasOwnProperty']['call']({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].prototype['hasOwnProperty'].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object'].prototype["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object']["prototype"].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object']["prototype"]["hasOwnProperty"]["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis['Object']['prototype']['hasOwnProperty']['call']({receiver_source}, {key_source})"#),
        format!("Object.prototype.hasOwnProperty.call({receiver_source}, {key_source})"),
        format!("globalThis.Object.prototype.hasOwnProperty.call({receiver_source}, {key_source})"),
        format!(r#"globalThis.Object.prototype.hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object["prototype"].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object["prototype"].hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object["prototype"]["hasOwnProperty"]["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis.Object.prototype["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"].prototype.hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"].prototype.hasOwnProperty["call"]({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"].prototype["hasOwnProperty"].call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"]["prototype"].hasOwnProperty.call({receiver_source}, {key_source})"#),
        format!(r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]({receiver_source}, {key_source})"#),
    ]
    .join("; ");
    format!("{source};")
}

/// Canonical late-process-control source prefix that groups the supported
/// Deno and process-control aliases before the shared zero-probe inventory.
const LATE_PROCESS_CONTROL_PREFIX_SEGMENTS: &[&str] = &[
    "Deno.pid",
    "globalThis.Deno.pid",
    "globalThis[\"Deno\"][\"pid\"]",
    "globalThis[\"Deno\"].cwd",
    "globalThis[\"Deno\"].chdir",
    "globalThis[\"Deno\"].exit",
    "Deno[\"pid\"]",
    "globalThis.Deno[\"pid\"]",
    "globalThis.Deno.cwd",
    "globalThis[\"Deno\"][\"cwd\"]",
    "globalThis.Deno[\"cwd\"]",
    "Deno[\"cwd\"]",
    "Deno.chdir",
    "globalThis.Deno.chdir",
    "globalThis[\"Deno\"][\"chdir\"]",
    "globalThis.Deno[\"chdir\"]",
    "Deno[\"chdir\"]",
    "globalThis.Deno.exit",
    "globalThis[\"Deno\"][\"exit\"]",
    "globalThis.Deno[\"exit\"]",
    "Deno[\"exit\"]",
    "process.pid",
    "globalThis.process.pid",
    "globalThis[\"process\"][\"pid\"]",
    "globalThis[\"process\"].pid",
    "process[\"pid\"]",
    "globalThis.process[\"pid\"]",
    "process.cwd",
    "globalThis.process.cwd",
    "globalThis[\"process\"].cwd",
    "globalThis[\"process\"][\"cwd\"]",
    "process[\"cwd\"]",
    "globalThis.process[\"cwd\"]",
    "process.chdir",
    "globalThis.process.chdir",
    "globalThis[\"process\"].chdir",
    "globalThis[\"process\"][\"chdir\"]",
    "process[\"chdir\"]",
    "globalThis.process[\"chdir\"]",
    "process.kill",
    "globalThis.process.kill",
    "globalThis[\"process\"].kill",
    "globalThis[\"process\"][\"kill\"]",
    "process[\"kill\"]",
    "globalThis.process[\"kill\"]",
    "const zero = 0",
    "const zeroAlias = zero",
    "process.kill(zeroAlias)",
];

const LATE_PROCESS_CONTROL_EXIT_SEGMENTS: &[&str] = &[
    "process.exit",
    "globalThis.process.exit",
    "globalThis.process[\"exit\"]",
    "globalThis[\"process\"].exit",
    "globalThis[\"process\"][\"exit\"]",
    "process[\"exit\"]",
];

/// Canonical exit alias inventory for the shared late-process-control slice.
pub const fn late_process_control_exit_aliases() -> &'static [&'static str] {
    LATE_PROCESS_CONTROL_EXIT_SEGMENTS
}

/// Canonical late-process-control exit source text, shared across the
/// browser and runtime late-compat smoke.
pub fn late_process_control_exit_source() -> String {
    join_semicolon_terminated_segments(late_process_control_exit_aliases())
}

/// Canonical late-process-control preamble source text, shared across the
/// browser and runtime late-compat smoke.
pub fn late_process_control_prefix_source() -> String {
    format!(
        "{}; {}",
        join_semicolon_terminated_segments(LATE_PROCESS_CONTROL_PREFIX_SEGMENTS)
            .trim_end_matches(';'),
        late_process_control_exit_source().trim_end()
    )
}

/// Canonical late-process-control source text that embeds the supported Node zero-probe slice.
pub fn late_process_control_source() -> String {
    let process_kill_zero_probe_source = process_kill_zero_probe_alias_inventory_source();
    let parenthesized_receiver_source = process_kill_zero_probe_parenthesized_receiver_source();
    let parenthesized_receiver_freeze_source =
        process_kill_zero_probe_parenthesized_receiver_freeze_source();
    format!(
        "{} {} {} {}",
        late_process_control_prefix_source(),
        parenthesized_receiver_source.trim_end(),
        parenthesized_receiver_freeze_source.trim_end(),
        process_kill_zero_probe_source.trim_end()
    )
}

const LATE_PROCESS_CONTROL_SINGLE_QUOTED_PROCESS_SEGMENTS: &[&str] = &[
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
];

/// Canonical late-process-control aliases for the browser JS single-quoted process root,
/// wrapped zero-literal, and exit aliases.
pub fn late_process_control_single_quoted_kill_aliases() -> &'static [&'static str] {
    &LATE_PROCESS_CONTROL_SINGLE_QUOTED_PROCESS_SEGMENTS[..33]
}

/// Canonical late-process-control source text for the browser JS single-quoted kill aliases.
pub fn late_process_control_single_quoted_kill_aliases_source() -> String {
    join_semicolon_terminated_segments(late_process_control_single_quoted_kill_aliases())
}

/// Canonical late-process-control aliases for the browser JS single-quoted process root,
/// wrapped zero-literal, and exit aliases.
pub fn late_process_control_single_quoted_exit_aliases() -> &'static [&'static str] {
    &LATE_PROCESS_CONTROL_SINGLE_QUOTED_PROCESS_SEGMENTS[33..]
}

/// Canonical late-process-control source text for the browser JS single-quoted exit aliases.
pub fn late_process_control_single_quoted_exit_source() -> String {
    join_semicolon_terminated_segments(late_process_control_single_quoted_exit_aliases())
}

/// Canonical late-process-control source text for the browser JS single-quoted exit aliases.
pub fn late_process_control_single_quoted_exit_aliases_source() -> String {
    late_process_control_single_quoted_exit_source()
}

/// Canonical late-process-control source text for the browser JS single-quoted kill aliases.
pub fn late_process_control_single_quoted_kill_source() -> String {
    late_process_control_single_quoted_kill_aliases_source()
}

/// Canonical late-process-control aliases for the browser JS single-quoted process root,
/// wrapped zero-literal, and exit aliases.
pub fn late_process_control_single_quoted_process_aliases() -> &'static [&'static str] {
    LATE_PROCESS_CONTROL_SINGLE_QUOTED_PROCESS_SEGMENTS
}

/// Canonical late-process-control source text for the browser JS single-quoted process root,
/// wrapped zero-literal, and exit aliases.
pub fn late_process_control_single_quoted_process_aliases_source() -> String {
    format!(
        "{} {}",
        late_process_control_single_quoted_kill_source().trim_end(),
        late_process_control_single_quoted_exit_source().trim_end()
    )
}

/// Canonical late-process-control source text for the browser JS single-quoted process root,
/// wrapped zero-literal, and exit aliases.
pub fn late_process_control_single_quoted_process_source() -> String {
    format!(
        "{} {}",
        late_process_control_source(),
        late_process_control_single_quoted_process_aliases_source().trim_end()
    )
}

const LATE_PROCESS_ENV_MUTATION_SEGMENTS: &[&str] = &[
    r#"process.env = {}"#,
    r#"process.env.KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis.process.env = {}"#,
    r#"globalThis.process.env.KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"process["env"] = {}"#,
    r#"process["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"process["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"process['env'] = {}"#,
    r#"process['env'].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"process['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
    r#"globalThis.process["env"] = {}"#,
    r#"globalThis.process["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"globalThis.process['env'] = {}"#,
    r#"globalThis.process['env'].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis.process['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
    r#"delete globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis.process['env']['KALI_BROWSER_ENV_MUTATION']"#,
    r#"globalThis["process"].env = {}"#,
    r#"globalThis["process"].env.KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"globalThis["process"]["env"] = {}"#,
    r#"globalThis["process"]["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"globalThis["process"]['env'] = {}"#,
    r#"globalThis["process"]['env'].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis["process"]['env']["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"globalThis["process"]['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
    r#"globalThis['process']["env"] = {}"#,
    r#"globalThis['process']["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis['process']["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
    r#"delete globalThis['process']["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis["process"]['env']['KALI_BROWSER_ENV_MUTATION']"#,
    r#"globalThis['process'].env = {}"#,
    r#"globalThis['process'].env.KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis['process'].env['KALI_BROWSER_ENV_MUTATION'] = {}"#,
    r#"globalThis['process']['env'] = {}"#,
    r#"globalThis['process']['env'].KALI_BROWSER_ENV_MUTATION = {}"#,
    r#"globalThis['process']['env']['KALI_BROWSER_ENV_MUTATION'] = {}"#,
    r#"delete process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete process['env']['KALI_BROWSER_ENV_MUTATION']"#,
    r#"delete process.env["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis.process['env']['KALI_BROWSER_ENV_MUTATION']"#,
    r#"delete globalThis.process.env["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    r#"delete globalThis['process'].env['KALI_BROWSER_ENV_MUTATION']"#,
    r#"delete globalThis['process']['env']['KALI_BROWSER_ENV_MUTATION']"#,
];

/// Canonical late-process-environment-mutation alias inventory used by the browser and runtime smoke.
pub fn late_process_env_mutation_aliases() -> &'static [&'static str] {
    LATE_PROCESS_ENV_MUTATION_SEGMENTS
}

/// Canonical late-process-environment-mutation source text used by the browser and runtime smoke.
pub fn late_process_env_mutation_source() -> String {
    join_semicolon_terminated_segments(late_process_env_mutation_aliases())
}

const LATE_OBJECT_MODEL_SEGMENTS: &[&str] = &[
    "Proxy",
    "globalThis.Proxy",
    r#"globalThis["Proxy"]"#,
    "globalThis['Proxy']",
    "new Proxy({}, {})",
    "new globalThis.Proxy({}, {})",
    r#"new globalThis["Proxy"]({}, {})"#,
    "new globalThis['Proxy']({}, {})",
    "new WeakMap()",
    "globalThis.WeakMap",
    r#"globalThis["WeakMap"]"#,
    r#"globalThis['WeakMap']"#,
    r#"globalThis["WeakMap"]()"#,
    r#"globalThis['WeakMap']()"#,
    "Object.freeze(new WeakMap())",
    "Object.freeze((new WeakMap()))",
    "Object.freeze(globalThis.WeakMap)",
    "Object.freeze((globalThis.WeakMap))",
    r#"Object.freeze(globalThis["WeakMap"])"#,
    r#"Object.freeze((globalThis["WeakMap"]))"#,
    r#"Object.freeze(globalThis['WeakMap'])"#,
    r#"Object.freeze((globalThis['WeakMap']))"#,
    "new WeakSet()",
    "globalThis.WeakSet",
    r#"globalThis["WeakSet"]"#,
    r#"globalThis['WeakSet']"#,
    r#"globalThis["WeakSet"]()"#,
    r#"globalThis['WeakSet']()"#,
    "Object.freeze(new WeakSet())",
    "Object.freeze((new WeakSet()))",
    "Object.freeze(globalThis.WeakSet)",
    "Object.freeze((globalThis.WeakSet))",
    r#"Object.freeze(globalThis["WeakSet"])"#,
    r#"Object.freeze((globalThis["WeakSet"]))"#,
    r#"Object.freeze(globalThis['WeakSet'])"#,
    r#"Object.freeze((globalThis['WeakSet']))"#,
    "globalThis.WeakRef",
    r#"globalThis["WeakRef"]"#,
    "globalThis['WeakRef']",
    "Object.freeze(globalThis.WeakRef)",
    "Object.freeze((globalThis.WeakRef))",
    r#"Object.freeze(globalThis["WeakRef"])"#,
    r#"Object.freeze((globalThis["WeakRef"]))"#,
    "Object.freeze(globalThis['WeakRef'])",
    "Object.freeze((globalThis['WeakRef']))",
    "new FinalizationRegistry(() => {})",
    "globalThis.FinalizationRegistry",
    r#"globalThis["FinalizationRegistry"](() => {})"#,
    r#"globalThis['FinalizationRegistry'](() => {})"#,
    "Object.freeze(new FinalizationRegistry(() => {}))",
    "Object.freeze((new FinalizationRegistry(() => {})))",
    "Object.freeze(globalThis.FinalizationRegistry)",
    "Object.freeze((globalThis.FinalizationRegistry))",
    r#"Object.freeze(globalThis["FinalizationRegistry"](() => {}))"#,
    r#"Object.freeze((globalThis["FinalizationRegistry"](() => {})))"#,
    r#"Object.freeze(globalThis['FinalizationRegistry'](() => {}))"#,
    r#"Object.freeze((globalThis['FinalizationRegistry'](() => {})))"#,
    r#"Object.freeze(globalThis["FinalizationRegistry"])"#,
    r#"Object.freeze((globalThis["FinalizationRegistry"]))"#,
    r#"Object.freeze(globalThis['FinalizationRegistry'])"#,
    r#"Object.freeze((globalThis['FinalizationRegistry']))"#,
    "Proxy.revocable({}, {})",
    "globalThis.Proxy.revocable({}, {})",
    r#"globalThis["Proxy"]["revocable"]({}, {})"#,
    r#"globalThis['Proxy']['revocable']({}, {})"#,
    r#"globalThis["Proxy"].revocable({}, {})"#,
    r#"globalThis['Proxy'].revocable({}, {})"#,
    r#"globalThis.Proxy["revocable"]({}, {})"#,
    r#"globalThis.Proxy['revocable']({}, {})"#,
    r#"globalThis['Proxy']["revocable"]({}, {})"#,
    r#"globalThis["Proxy"]['revocable']({}, {})"#,
    r#"Object.freeze(globalThis['Proxy']["revocable"])({}, {})"#,
    r#"Object.freeze((globalThis['Proxy']["revocable"]))({}, {})"#,
    r#"Object.freeze((globalThis["Proxy"])["revocable"])({}, {})"#,
    r#"Object.freeze((globalThis['Proxy'])['revocable'])({}, {})"#,
    r#"Object.freeze(globalThis["Proxy"]['revocable'])({}, {})"#,
    r#"Object.freeze((globalThis["Proxy"]['revocable']))({}, {})"#,
    "Object.freeze(Proxy.revocable)({}, {})",
    "Object.freeze((Proxy.revocable))({}, {})",
    "Object.freeze(globalThis.Proxy.revocable)({}, {})",
    "Object.freeze((globalThis.Proxy.revocable))({}, {})",
    r#"Object.freeze(globalThis["Proxy"]["revocable"])({}, {})"#,
    r#"Object.freeze((globalThis["Proxy"]["revocable"]))({}, {})"#,
    r#"Object.freeze(globalThis['Proxy']['revocable'])({}, {})"#,
    r#"Object.freeze((globalThis['Proxy']['revocable']))({}, {})"#,
    r#"Object.freeze(globalThis["Proxy"].revocable)({}, {})"#,
    r#"Object.freeze((globalThis["Proxy"].revocable))({}, {})"#,
    r#"Object.freeze(globalThis['Proxy'].revocable)({}, {})"#,
    r#"Object.freeze((globalThis['Proxy']).revocable)({}, {})"#,
    r#"Object.freeze((globalThis['Proxy'].revocable))({}, {})"#,
    r#"Object.freeze(globalThis.Proxy["revocable"])({}, {})"#,
    r#"Object.freeze((globalThis.Proxy["revocable"]))({}, {})"#,
    r#"Object.freeze(globalThis.Proxy['revocable'])({}, {})"#,
    r#"Object.freeze((globalThis.Proxy['revocable']))({}, {})"#,
];

/// Canonical alias inventory for the shared late-object-model slice.
pub const fn late_object_model_aliases() -> &'static [&'static str] {
    LATE_OBJECT_MODEL_SEGMENTS
}

/// Canonical late-object-model source text used by the browser and runtime smoke.
pub const fn late_object_model_source() -> &'static str {
    r#"Proxy; globalThis.Proxy; globalThis["Proxy"]; globalThis['Proxy']; new Proxy({}, {}); new globalThis.Proxy({}, {}); new globalThis["Proxy"]({}, {}); new globalThis['Proxy']({}, {}); new WeakMap(); globalThis.WeakMap; globalThis["WeakMap"]; globalThis['WeakMap']; globalThis["WeakMap"](); globalThis['WeakMap'](); Object.freeze(new WeakMap()); Object.freeze((new WeakMap())); Object.freeze(globalThis.WeakMap); Object.freeze((globalThis.WeakMap)); Object.freeze(globalThis["WeakMap"]); Object.freeze((globalThis["WeakMap"])); Object.freeze(globalThis['WeakMap']); Object.freeze((globalThis['WeakMap'])); new WeakSet(); globalThis.WeakSet; globalThis["WeakSet"]; globalThis['WeakSet']; globalThis["WeakSet"](); globalThis['WeakSet'](); Object.freeze(new WeakSet()); Object.freeze((new WeakSet())); Object.freeze(globalThis.WeakSet); Object.freeze((globalThis.WeakSet)); Object.freeze(globalThis["WeakSet"]); Object.freeze((globalThis["WeakSet"])); Object.freeze(globalThis['WeakSet']); Object.freeze((globalThis['WeakSet'])); globalThis.WeakRef; globalThis["WeakRef"]; globalThis['WeakRef']; Object.freeze(globalThis.WeakRef); Object.freeze((globalThis.WeakRef)); Object.freeze(globalThis["WeakRef"]); Object.freeze((globalThis["WeakRef"])); Object.freeze(globalThis['WeakRef']); Object.freeze((globalThis['WeakRef'])); new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry; globalThis["FinalizationRegistry"](() => {}); globalThis['FinalizationRegistry'](() => {}); Object.freeze(new FinalizationRegistry(() => {})); Object.freeze((new FinalizationRegistry(() => {}))); Object.freeze(globalThis.FinalizationRegistry); Object.freeze((globalThis.FinalizationRegistry)); Object.freeze(globalThis["FinalizationRegistry"](() => {})); Object.freeze((globalThis["FinalizationRegistry"](() => {}))); Object.freeze(globalThis['FinalizationRegistry'](() => {})); Object.freeze((globalThis['FinalizationRegistry'](() => {}))); Object.freeze(globalThis["FinalizationRegistry"]); Object.freeze((globalThis["FinalizationRegistry"])); Object.freeze(globalThis['FinalizationRegistry']); Object.freeze((globalThis['FinalizationRegistry'])); Proxy.revocable({}, {}); globalThis.Proxy.revocable({}, {}); globalThis["Proxy"]["revocable"]({}, {}); globalThis['Proxy']['revocable']({}, {}); globalThis["Proxy"].revocable({}, {}); globalThis['Proxy'].revocable({}, {}); globalThis.Proxy["revocable"]({}, {}); globalThis.Proxy['revocable']({}, {}); globalThis['Proxy']["revocable"]({}, {}); globalThis["Proxy"]['revocable']({}, {}); Object.freeze(globalThis['Proxy']["revocable"])({}, {}); Object.freeze((globalThis['Proxy']["revocable"]))({}, {}); Object.freeze((globalThis["Proxy"])["revocable"])({}, {}); Object.freeze((globalThis['Proxy'])['revocable'])({}, {}); Object.freeze(globalThis["Proxy"]['revocable'])({}, {}); Object.freeze((globalThis["Proxy"]['revocable']))({}, {}); Object.freeze(Proxy.revocable)({}, {}); Object.freeze((Proxy.revocable))({}, {}); Object.freeze(globalThis.Proxy.revocable)({}, {}); Object.freeze((globalThis.Proxy.revocable))({}, {}); Object.freeze(globalThis["Proxy"]["revocable"])({}, {}); Object.freeze((globalThis["Proxy"]["revocable"]))({}, {}); Object.freeze(globalThis['Proxy']['revocable'])({}, {}); Object.freeze((globalThis['Proxy']['revocable']))({}, {}); Object.freeze(globalThis["Proxy"].revocable)({}, {}); Object.freeze((globalThis["Proxy"].revocable))({}, {}); Object.freeze(globalThis['Proxy'].revocable)({}, {}); Object.freeze((globalThis['Proxy']).revocable)({}, {}); Object.freeze((globalThis['Proxy'].revocable))({}, {}); Object.freeze(globalThis.Proxy["revocable"])({}, {}); Object.freeze((globalThis.Proxy["revocable"]))({}, {}); Object.freeze(globalThis.Proxy['revocable'])({}, {}); Object.freeze((globalThis.Proxy['revocable']))({}, {});"#
}

const LATE_OBJECT_MODEL_OWN_PROPERTY_SEGMENTS: &[&str] = &[
    r#"Object.hasOwn(globalThis, "a")"#,
    r#"globalThis.Object.hasOwn(globalThis, "a")"#,
    r#"globalThis.Object["hasOwn"](globalThis, "a")"#,
    r#"globalThis["Object"].hasOwn(globalThis, "a")"#,
    r#"globalThis["Object"]["hasOwn"](globalThis, "a")"#,
    r#"Object["hasOwnProperty"].call(globalThis, "a")"#,
    r#"globalThis.Object["hasOwnProperty"].call(globalThis, "a")"#,
    r#"globalThis.Object['hasOwnProperty'].call(globalThis, "a")"#,
    r#"globalThis["Object"]["hasOwnProperty"].call(globalThis, "a")"#,
    r#"globalThis["Object"]['hasOwnProperty'].call(globalThis, "a")"#,
    r#"Object.prototype.hasOwnProperty.call(globalThis, "a")"#,
    r#"globalThis.Object.prototype.hasOwnProperty.call(globalThis, "a")"#,
    r#"globalThis.Object.prototype.hasOwnProperty["call"](globalThis, "a")"#,
    r#"globalThis.Object["prototype"].hasOwnProperty.call(globalThis, "a")"#,
    r#"globalThis.Object["prototype"]["hasOwnProperty"]["call"](globalThis, "a")"#,
    r#"globalThis.Object.prototype["hasOwnProperty"].call(globalThis, "a")"#,
    r#"globalThis["Object"].prototype.hasOwnProperty.call(globalThis, "a")"#,
    r#"globalThis["Object"].prototype.hasOwnProperty["call"](globalThis, "a")"#,
    r#"globalThis["Object"].prototype['hasOwnProperty']['call'](globalThis, "a")"#,
    r#"globalThis["Object"].prototype['hasOwnProperty'].call(globalThis, "a")"#,
    r#"globalThis["Object"].prototype["hasOwnProperty"].call(globalThis, "a")"#,
    r#"globalThis["Object"]["prototype"].hasOwnProperty.call(globalThis, "a")"#,
    r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](globalThis, "a")"#,
    r#"globalThis["Object"]["prototype"].hasOwnProperty["call"](globalThis, "a")"#,
    r#"globalThis.Object["prototype"].hasOwnProperty["call"](globalThis, "a")"#,
];

/// Canonical alias inventory for the shared late-object-model own-property slice.
pub fn late_object_model_own_property_aliases() -> &'static [&'static str] {
    LATE_OBJECT_MODEL_OWN_PROPERTY_SEGMENTS
}

/// Canonical late-object-model own-property source text used by the browser and runtime smoke.
pub const fn late_object_model_own_property_source() -> &'static str {
    r#"Object.hasOwn(globalThis, "a"); globalThis.Object.hasOwn(globalThis, "a"); globalThis.Object["hasOwn"](globalThis, "a"); globalThis["Object"].hasOwn(globalThis, "a"); globalThis["Object"]["hasOwn"](globalThis, "a"); Object["hasOwnProperty"].call(globalThis, "a"); globalThis.Object["hasOwnProperty"].call(globalThis, "a"); globalThis.Object['hasOwnProperty'].call(globalThis, "a"); globalThis["Object"]["hasOwnProperty"].call(globalThis, "a"); globalThis["Object"]['hasOwnProperty'].call(globalThis, "a"); Object.prototype.hasOwnProperty.call(globalThis, "a"); globalThis.Object.prototype.hasOwnProperty.call(globalThis, "a"); globalThis.Object.prototype.hasOwnProperty["call"](globalThis, "a"); globalThis.Object["prototype"].hasOwnProperty.call(globalThis, "a"); globalThis.Object["prototype"]["hasOwnProperty"]["call"](globalThis, "a"); globalThis.Object.prototype["hasOwnProperty"].call(globalThis, "a"); globalThis["Object"].prototype.hasOwnProperty.call(globalThis, "a"); globalThis["Object"].prototype.hasOwnProperty["call"](globalThis, "a"); globalThis["Object"].prototype['hasOwnProperty']['call'](globalThis, "a"); globalThis["Object"].prototype['hasOwnProperty'].call(globalThis, "a"); globalThis["Object"].prototype["hasOwnProperty"].call(globalThis, "a"); globalThis["Object"]["prototype"].hasOwnProperty.call(globalThis, "a"); globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](globalThis, "a"); globalThis["Object"]["prototype"].hasOwnProperty["call"](globalThis, "a"); globalThis.Object["prototype"].hasOwnProperty["call"](globalThis, "a");"#
}

const LATE_THREADED_RUNTIME_SEGMENTS: &[&str] = &[
    "SharedArrayBuffer",
    "globalThis.SharedArrayBuffer",
    r#"globalThis["SharedArrayBuffer"]"#,
    "globalThis['SharedArrayBuffer']",
    "Object.freeze(globalThis.SharedArrayBuffer)",
    r#"Object.freeze(globalThis["SharedArrayBuffer"])"#,
    "Object.freeze(globalThis['SharedArrayBuffer'])",
    "Object.freeze(SharedArrayBuffer)",
    "Object.freeze((SharedArrayBuffer))",
    "Object.freeze((globalThis.SharedArrayBuffer))",
    r#"Object.freeze((globalThis["SharedArrayBuffer"]))"#,
    "Object.freeze((globalThis['SharedArrayBuffer']))",
    "Object.freeze((null ?? globalThis.SharedArrayBuffer))",
    "Object.freeze((null ?? globalThis['SharedArrayBuffer']))",
    r#"Object.freeze((true && globalThis["SharedArrayBuffer"]))"#,
    "Object.freeze((true && globalThis['SharedArrayBuffer']))",
    "Object.freeze((true && globalThis.SharedArrayBuffer))",
    r#"Object.freeze((false || globalThis["SharedArrayBuffer"]))"#,
    "Object.freeze((false || globalThis['SharedArrayBuffer']))",
    "Object.freeze((false || globalThis.SharedArrayBuffer))",
    "Atomics",
    "globalThis.Atomics",
    r#"globalThis["Atomics"]"#,
    "globalThis['Atomics']",
    "Object.freeze(globalThis.Atomics)",
    r#"Object.freeze(globalThis["Atomics"])"#,
    "Object.freeze(globalThis['Atomics'])",
    "Object.freeze(Atomics)",
    "Object.freeze((Atomics))",
    "Object.freeze((globalThis.Atomics))",
    r#"Object.freeze((globalThis["Atomics"]))"#,
    "Object.freeze((globalThis['Atomics']))",
    "Object.freeze((null ?? globalThis.Atomics))",
    "Object.freeze((null ?? globalThis['Atomics']))",
    r#"Object.freeze((true && globalThis["Atomics"]))"#,
    "Object.freeze((true && globalThis['Atomics']))",
    "Object.freeze((true && globalThis.Atomics))",
    r#"Object.freeze((false || globalThis["Atomics"]))"#,
    "Object.freeze((false || globalThis['Atomics']))",
    "Object.freeze((false || globalThis.Atomics))",
];

/// Canonical alias inventory for the shared late-threaded-runtime slice.
pub fn late_threaded_runtime_aliases() -> &'static [&'static str] {
    LATE_THREADED_RUNTIME_SEGMENTS
}

/// Canonical late-threaded-runtime source text used by the browser and runtime smoke.
pub const fn late_threaded_runtime_source() -> &'static str {
    "SharedArrayBuffer; globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis['SharedArrayBuffer']; Object.freeze(globalThis.SharedArrayBuffer); Object.freeze(globalThis[\"SharedArrayBuffer\"]); Object.freeze(globalThis['SharedArrayBuffer']); Object.freeze(SharedArrayBuffer); Object.freeze((SharedArrayBuffer)); Object.freeze((globalThis.SharedArrayBuffer)); Object.freeze((globalThis[\"SharedArrayBuffer\"])); Object.freeze((globalThis['SharedArrayBuffer'])); Object.freeze((null ?? globalThis.SharedArrayBuffer)); Object.freeze((null ?? globalThis['SharedArrayBuffer'])); Object.freeze((true && globalThis[\"SharedArrayBuffer\"])); Object.freeze((true && globalThis['SharedArrayBuffer'])); Object.freeze((true && globalThis.SharedArrayBuffer)); Object.freeze((false || globalThis[\"SharedArrayBuffer\"])); Object.freeze((false || globalThis['SharedArrayBuffer'])); Object.freeze((false || globalThis.SharedArrayBuffer)); Atomics; globalThis.Atomics; globalThis[\"Atomics\"]; globalThis['Atomics']; Object.freeze(globalThis.Atomics); Object.freeze(globalThis[\"Atomics\"]); Object.freeze(globalThis['Atomics']); Object.freeze(Atomics); Object.freeze((Atomics)); Object.freeze((globalThis.Atomics)); Object.freeze((globalThis[\"Atomics\"])); Object.freeze((globalThis['Atomics'])); Object.freeze((null ?? globalThis.Atomics)); Object.freeze((null ?? globalThis['Atomics'])); Object.freeze((true && globalThis[\"Atomics\"])); Object.freeze((true && globalThis['Atomics'])); Object.freeze((true && globalThis.Atomics)); Object.freeze((false || globalThis[\"Atomics\"])); Object.freeze((false || globalThis['Atomics'])); Object.freeze((false || globalThis.Atomics));"
}

const LATE_PERMISSION_ESCALATION_SEGMENTS: &[&str] = &[
    "Deno.permissions.request()",
    "Deno.permissions.revoke()",
    r#"Deno.permissions["request"]()"#,
    r#"Deno.permissions["revoke"]()"#,
    "globalThis.Deno.permissions.request()",
    "globalThis.Deno.permissions.revoke()",
    r#"globalThis.Deno.permissions["request"]()"#,
    r#"globalThis.Deno.permissions["revoke"]()"#,
    r#"globalThis["Deno"].permissions["request"]()"#,
    r#"globalThis["Deno"].permissions["revoke"]()"#,
    r#"globalThis["Deno"].permissions.request()"#,
    r#"globalThis["Deno"].permissions.revoke()"#,
    r#"globalThis["Deno"].permissions["request"]()"#,
    r#"globalThis["Deno"]["permissions"]["request"]()"#,
    r#"globalThis["Deno"]["permissions"]["revoke"]()"#,
    r#"globalThis["Deno"]["permissions"].request()"#,
    r#"globalThis["Deno"]["permissions"].revoke()"#,
    r#"globalThis["Deno"].permissions["request"]()"#,
    r#"globalThis["Deno"].permissions["revoke"]()"#,
    r#"globalThis.Deno["permissions"]["request"]()"#,
    r#"globalThis.Deno["permissions"]["revoke"]()"#,
];

/// Canonical late permission-escalation alias inventory used by the browser and runtime smoke.
pub fn late_permission_escalation_aliases() -> &'static [&'static str] {
    LATE_PERMISSION_ESCALATION_SEGMENTS
}

/// Canonical late permission-escalation source text used by the browser and runtime smoke.
pub fn late_permission_escalation_source() -> String {
    join_semicolon_terminated_segments(late_permission_escalation_aliases())
}

/// Canonical late environment-materialization source text used by the browser and runtime smoke.
pub const fn late_env_materialization_source() -> &'static str {
    "Deno.env.toObject(); globalThis.Deno.env.toObject(); Deno.env[\"toObject\"](); Deno[\"env\"][\"toObject\"](); Deno[\"env\"].toObject(); globalThis.Deno.env[\"toObject\"](); globalThis.Deno[\"env\"][\"toObject\"](); globalThis.Deno[\"env\"].toObject(); globalThis[\"Deno\"].env.toObject(); globalThis[\"Deno\"].env[\"toObject\"](); globalThis[\"Deno\"][\"env\"].toObject(); globalThis[\"Deno\"][\"env\"][\"toObject\"](); globalThis.Deno[\"env\"][\"toObject\"](); globalThis[\"Deno\"].env.toObject();"
}

/// Canonical late subprocess source text used by the browser and runtime smoke.
pub const fn late_subprocess_source() -> &'static str {
    "new Deno.Command('sh').spawn(); new Deno[\"Command\"]('sh').spawn(); new globalThis.Deno.Command('sh').spawn(); new globalThis.Deno[\"Command\"]('sh').spawn(); new globalThis[\"Deno\"].Command('sh').spawn(); new globalThis[\"Deno\"][\"Command\"]('sh').spawn();"
}

/// Canonical late network source text used by the browser and runtime smoke.
pub const fn late_network_source() -> &'static str {
    "Deno.connect('127.0.0.1', 1); globalThis.Deno.connect('127.0.0.1', 1); globalThis.Deno[\"connect\"]('127.0.0.1', 1); globalThis[\"Deno\"].connect('127.0.0.1', 1); globalThis[\"Deno\"][\"connect\"]('127.0.0.1', 1); Deno.listen('127.0.0.1', 0); globalThis.Deno.listen('127.0.0.1', 0); globalThis.Deno[\"listen\"]('127.0.0.1', 0); globalThis[\"Deno\"].listen('127.0.0.1', 0); globalThis[\"Deno\"][\"listen\"]('127.0.0.1', 0); Deno.serve('127.0.0.1', 0); globalThis.Deno.serve('127.0.0.1', 0); globalThis.Deno[\"serve\"]('127.0.0.1', 0); globalThis[\"Deno\"].serve('127.0.0.1', 0); globalThis[\"Deno\"][\"serve\"]('127.0.0.1', 0);"
}

#[cfg(test)]
#[path = "late_tests.rs"]
mod late_tests;
