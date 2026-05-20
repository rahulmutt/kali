use super::*;

#[test]
fn test_file_id_basic() {
    let fid = FileId::new(42);
    assert_eq!(fid.as_u32(), 42);
    assert_eq!(fid.to_string(), "f42");
}

#[test]
fn test_source_file() {
    let sf = SourceFile::new(FileId::new(0), "/path/to/file.ts");
    assert_eq!(sf.filename(), "file.ts");
    assert_eq!(sf.extension(), "ts");
    assert_eq!(sf.directory(), "/path/to");
}

#[test]
fn test_source_registry_interning() {
    let mut registry = SourceRegistry::default();

    let path = Path::new("/test/file.ts");
    let fid1 = registry.intern_path(path);
    let fid2 = registry.intern_path(path);

    // Same path should give same ID
    assert_eq!(fid1, fid2);

    // Different paths should give different IDs
    let fid3 = registry.intern_path(Path::new("/test/other.ts"));
    assert_ne!(fid1, fid3);
}

#[test]
fn test_bytewise_shared_memory_lock_free_probe_matches_target_atomic_support() {
    assert_eq!(
        bytewise_shared_memory_is_lock_free(),
        cfg!(target_has_atomic = "8")
    );
}

#[test]
fn test_late_object_model_source_lists_proxy_and_weak_aliases() {
    let source = late_object_model_source();
    assert!(source.contains("Proxy.revocable"), "source: {source}");
    assert!(
        source.contains(r#"Object.freeze(Proxy.revocable)"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis.Proxy.revocable)"#),
        "source: {source}"
    );
    assert!(source.contains("WeakMap"), "source: {source}");
    assert!(source.contains("WeakSet"), "source: {source}");
    assert!(source.contains("WeakRef"), "source: {source}");
    assert!(source.contains("FinalizationRegistry"), "source: {source}");
}

#[test]
fn test_late_object_model_own_property_source_lists_shared_helper_family() {
    let source = late_object_model_own_property_source();
    assert!(
        source.contains("Object.hasOwn(globalThis, \"a\")"),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object["hasOwnProperty"].call(globalThis, "a")"#),
        "source: {source}"
    );
    assert!(
        source.contains(
            r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](globalThis, "a")"#
        ),
        "source: {source}"
    );
}

#[test]
fn test_late_threaded_runtime_source_lists_bracketed_spellings() {
    let source = late_threaded_runtime_source();
    assert!(
        source.contains(r#"globalThis["SharedArrayBuffer"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Atomics"]"#),
        "source: {source}"
    );
}

#[test]
fn test_late_permission_escalation_source_lists_request_and_revoke_aliases() {
    assert_eq!(
        late_permission_escalation_aliases(),
        &[
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
        ]
    );
    assert_eq!(
        late_permission_escalation_source(),
        r#"Deno.permissions.request(); Deno.permissions.revoke(); Deno.permissions["request"](); Deno.permissions["revoke"](); globalThis.Deno.permissions.request(); globalThis.Deno.permissions.revoke(); globalThis.Deno.permissions["request"](); globalThis.Deno.permissions["revoke"](); globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions["revoke"](); globalThis["Deno"].permissions.request(); globalThis["Deno"].permissions.revoke(); globalThis["Deno"].permissions["request"](); globalThis["Deno"]["permissions"]["request"](); globalThis["Deno"]["permissions"]["revoke"](); globalThis["Deno"]["permissions"].request(); globalThis["Deno"]["permissions"].revoke(); globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions["revoke"](); globalThis.Deno["permissions"]["request"](); globalThis.Deno["permissions"]["revoke"]();"#
    );
}

#[test]
fn test_late_env_materialization_source_lists_to_object_aliases() {
    assert_eq!(
        late_env_materialization_source(),
        r#"Deno.env.toObject(); globalThis.Deno.env.toObject(); Deno.env["toObject"](); Deno["env"]["toObject"](); Deno["env"].toObject(); globalThis.Deno.env["toObject"](); globalThis.Deno["env"]["toObject"](); globalThis.Deno["env"].toObject(); globalThis["Deno"].env.toObject(); globalThis["Deno"].env["toObject"](); globalThis["Deno"]["env"].toObject(); globalThis["Deno"]["env"]["toObject"](); globalThis.Deno["env"]["toObject"](); globalThis["Deno"].env.toObject();"#
    );
}

#[test]
fn test_late_subprocess_source_lists_command_aliases() {
    assert_eq!(
        late_subprocess_source(),
        r#"new Deno.Command('sh').spawn(); new globalThis.Deno.Command('sh').spawn(); new globalThis.Deno["Command"]('sh').spawn(); new globalThis["Deno"].Command('sh').spawn(); new globalThis["Deno"]["Command"]('sh').spawn();"#
    );
}

#[test]
fn test_late_network_source_lists_connect_listen_and_serve_aliases() {
    assert_eq!(
        late_network_source(),
        r#"Deno.connect('127.0.0.1', 1); globalThis.Deno.connect('127.0.0.1', 1); globalThis.Deno["connect"]('127.0.0.1', 1); globalThis["Deno"].connect('127.0.0.1', 1); globalThis["Deno"]["connect"]('127.0.0.1', 1); Deno.listen('127.0.0.1', 0); globalThis.Deno.listen('127.0.0.1', 0); globalThis.Deno["listen"]('127.0.0.1', 0); globalThis["Deno"].listen('127.0.0.1', 0); globalThis["Deno"]["listen"]('127.0.0.1', 0); Deno.serve('127.0.0.1', 0); globalThis.Deno.serve('127.0.0.1', 0); globalThis.Deno["serve"]('127.0.0.1', 0); globalThis["Deno"].serve('127.0.0.1', 0); globalThis["Deno"]["serve"]('127.0.0.1', 0);"#
    );
}

#[test]
fn test_async_class_method_lowering_unavailable_message_is_stable() {
    assert_eq!(
        async_class_method_lowering_unavailable_message(),
        "async class method lowering is unavailable in the direct runtime path; use a plain method or the later compatibility path"
    );
}

#[test]
fn test_generator_class_method_lowering_unavailable_message_lists_async_and_sync_variants() {
    assert_eq!(
        generator_class_method_lowering_unavailable_message(false),
        "generator class method lowering is unavailable in the direct runtime path; use a plain or async method, or the later compatibility path"
    );
    assert_eq!(
        generator_class_method_lowering_unavailable_message(true),
        "async-generator class method lowering is unavailable in the direct runtime path; use a plain or async method, or the later compatibility path"
    );
}

#[test]
fn test_generator_class_method_lowering_unavailable_message_for_flavors_is_stable() {
    const BOTH: &str = generator_class_method_lowering_unavailable_message_for_flavors(true, true);

    assert_eq!(
        BOTH,
        "generator and async-generator class method lowering is unavailable in the direct runtime path; use a plain or async method, or the later compatibility path"
    );
    assert_eq!(
        generator_class_method_lowering_unavailable_message_for_flavors(true, false),
        generator_class_method_lowering_unavailable_message(false)
    );
    assert_eq!(
        generator_class_method_lowering_unavailable_message_for_flavors(false, true),
        generator_class_method_lowering_unavailable_message(true)
    );
    assert_eq!(
        generator_class_method_lowering_unavailable_message_for_flavors(false, false),
        generator_class_method_lowering_unavailable_message(false)
    );
}

#[test]
fn test_generator_function_lowering_unavailable_message_lists_async_and_sync_variants() {
    assert_eq!(
        generator_function_lowering_unavailable_message(false),
        "generator function lowering is unavailable in the current phase; use a synchronous function or the later compatibility path"
    );
    assert_eq!(
        generator_function_lowering_unavailable_message(true),
        "async-generator function lowering is unavailable in the current phase; use a synchronous function or the later compatibility path"
    );
}

#[test]
fn test_generator_function_lowering_unavailable_message_for_flavors_is_stable() {
    const BOTH: &str = generator_function_lowering_unavailable_message_for_flavors(true, true);

    assert_eq!(
        BOTH,
        "generator and async-generator function lowering is unavailable in the current phase; use a synchronous function or the later compatibility path"
    );
    assert_eq!(
        generator_function_lowering_unavailable_message_for_flavors(true, false),
        generator_function_lowering_unavailable_message(false)
    );
    assert_eq!(
        generator_function_lowering_unavailable_message_for_flavors(false, true),
        generator_function_lowering_unavailable_message(true)
    );
    assert_eq!(
        generator_function_lowering_unavailable_message_for_flavors(false, false),
        generator_function_lowering_unavailable_message(false)
    );
}

#[test]
fn test_array_from_aliases_list_all_supported_aliases_in_order() {
    let aliases = array_from_aliases();
    let source = array_from_source();
    let expected = format!("{};", aliases.join("; "));

    assert_eq!(
        aliases,
        &[
            "Array.from",
            "globalThis.Array.from",
            r#"globalThis["Array"].from"#,
            r#"globalThis["Array"]["from"]"#,
            r#"globalThis['Array'].from"#,
            r#"globalThis['Array']['from']"#,
            r#"Array["from"]"#,
            r#"Array['from']"#,
            r#"globalThis.Array["from"]"#,
            r#"globalThis.Array['from']"#,
        ]
    );

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Array.from inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, expected);
}

#[test]
fn test_array_from_frozen_callable_aliases_list_all_supported_aliases_in_order() {
    let aliases = array_from_frozen_callable_aliases();
    let source = array_from_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    assert_eq!(
        aliases,
        &[
            r#"Object.freeze(Array.from)"#,
            r#"Object.freeze((Array.from))"#,
            r#"Object.freeze(globalThis.Array.from)"#,
            r#"Object.freeze((globalThis.Array.from))"#,
            r#"Object.freeze(globalThis["Array"].from)"#,
            r#"Object.freeze((globalThis["Array"].from))"#,
            r#"Object.freeze((globalThis["Array"]).from)"#,
            r#"Object.freeze((globalThis["Array"])["from"])"#,
            r#"Object.freeze(globalThis["Array"]["from"])"#,
            r#"Object.freeze((globalThis["Array"]["from"]))"#,
            r#"Object.freeze(globalThis['Array'].from)"#,
            r#"Object.freeze((globalThis['Array'].from))"#,
            r#"Object.freeze((globalThis['Array']).from)"#,
            r#"Object.freeze((globalThis['Array'])["from"])"#,
            r#"Object.freeze((globalThis["Array"]))["from"]"#,
            r#"Object.freeze((globalThis['Array']))["from"]"#,
            r#"Object.freeze(globalThis['Array']['from'])"#,
            r#"Object.freeze((globalThis['Array']['from']))"#,
            r#"Object.freeze(Array['from'])"#,
            r#"Object.freeze((Array['from']))"#,
            r#"Object.freeze(Array["from"])"#,
            r#"Object.freeze((Array["from"]))"#,
            r#"Object.freeze(globalThis.Array['from'])"#,
            r#"Object.freeze((globalThis.Array['from']))"#,
            r#"Object.freeze(globalThis.Array["from"])"#,
            r#"Object.freeze((globalThis.Array["from"]))"#,
        ]
    );

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Array.from frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, expected);
}

#[test]
fn test_array_from_alias_inventory_source_reuses_the_shared_helper_sources_once() {
    let source = array_from_alias_inventory_source();
    assert_eq!(
        source,
        format!(
            "{} {}",
            array_from_source().trim_end(),
            array_from_frozen_callable_source().trim_end()
        )
    );
    assert_eq!(
        source.matches(&array_from_source()).count(),
        1,
        "source: {source}"
    );
    assert_eq!(
        source.matches(&array_from_frozen_callable_source()).count(),
        1,
        "source: {source}"
    );
}

#[test]
fn test_array_from_loop_lines_renders_all_aliases_in_order() {
    let source = array_from_loop_lines(
        "Array.from; globalThis.Array.from",
        "for (const value of ",
        "  ",
    );
    assert_eq!(
        source,
        "  for (const value of Array.from(values)) {\n    console.log(value);\n  }\n  for (const value of globalThis.Array.from(values)) {\n    console.log(value);\n  }"
    );
}

#[test]
fn test_template_literal_string_iteration_body_source_is_canonical() {
    assert_eq!(
        template_literal_string_iteration_body_source(),
        "for (const ch of `hello`) { console.log(ch); }"
    );
}

#[test]
fn test_browser_template_literal_string_iteration_body_source_is_canonical() {
    assert_eq!(
        browser_template_literal_string_iteration_body_source(),
        concat!(
            "const prefix = \"he\";\n",
            "const suffix = \"llo\";\n",
            "const syncChars = [];\n",
            "for (const item of `${prefix}${suffix}`) {\n",
            "  syncChars.push(item);\n",
            "}\n",
            "const asyncChars = [];\n",
            "for await (const item of `${prefix}${suffix}`) {\n",
            "  asyncChars.push(item);\n",
            "}\n",
            "if (syncChars.join(\"\") !== \"hello\" || asyncChars.join(\"\") !== \"hello\") {\n",
            "  throw new Error('unexpected template literal iteration semantics');\n",
            "}"
        )
    );
}

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
fn test_reflect_own_keys_frozen_callable_aliases_list_all_aliases_in_order() {
    let aliases = reflect_own_keys_frozen_callable_aliases();

    assert_eq!(
        aliases,
        &[
            r#"Object.freeze(globalThis.Reflect.ownKeys)"#,
            r#"Object.freeze(globalThis.Reflect["ownKeys"])"#,
            r#"Object.freeze(globalThis["Reflect"].ownKeys)"#,
            r#"Object.freeze(globalThis["Reflect"]["ownKeys"])"#,
            r#"Object.freeze((globalThis.Reflect["ownKeys"]))"#,
            r#"Object.freeze((globalThis["Reflect"].ownKeys))"#,
            r#"Object.freeze((globalThis["Reflect"]["ownKeys"]))"#,
            r#"Object.freeze((globalThis.Reflect.ownKeys))"#,
        ]
    );

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Reflect.ownKeys frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
}

#[test]
fn test_reflect_own_keys_frozen_callable_source_lists_all_aliases_in_order() {
    let source = reflect_own_keys_frozen_callable_source("obj");
    let expected = concat!(
        "const frozenCallableKeys = Object.freeze(globalThis.Reflect.ownKeys)(obj); ",
        "const frozenMixedBracketedKeys = Object.freeze(globalThis.Reflect[\"ownKeys\"])(obj); ",
        "const frozenMixedRootKeys = Object.freeze(globalThis[\"Reflect\"].ownKeys)(obj); ",
        "const frozenBracketedKeys = Object.freeze(globalThis[\"Reflect\"][\"ownKeys\"])(obj); ",
        "const parenthesizedFrozenMixedBracketedKeys = Object.freeze((globalThis.Reflect[\"ownKeys\"]))(obj); ",
        "const parenthesizedFrozenMixedRootKeys = Object.freeze((globalThis[\"Reflect\"].ownKeys))(obj); ",
        "const parenthesizedFrozenBracketedKeys = Object.freeze((globalThis[\"Reflect\"][\"ownKeys\"]))(obj); ",
        "const parenthesizedFrozenCallableKeys = Object.freeze((globalThis.Reflect.ownKeys))(obj);"
    );

    assert_eq!(source, expected);
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
        "const zero = 0; const zeroAlias = zero; {} {} {} {} console.log(process.kill(zeroAlias)); console.log(dotRootKill(+zero)); console.log(globalThis[\"process\"][\"kill\"](zero)); console.log(process[\"kill\"](zero)); console.log(kill(0)); console.log(bracketedDotKill(+0)); console.log(dotBracketKill(0)); console.log(fullyBracketedKill(0)); console.log(sequenceKill(0)); console.log(bracketedRootSequenceKill(0)); console.log(dotRootSequenceKill(0)); console.log(bracketedSequenceKill(0)); console.log(dotBracketSequenceKill(0)); console.log(bracketedDotSequenceKill(0)); console.log(((globalThis[\"process\"][\"kill\"]))(+0));\n",
        process_kill_zero_probe_call_target_bindings_source(),
        process_kill_zero_probe_sequence_call_target_bindings_source(),
        process_kill_zero_probe_parenthesized_receiver_freeze_source(),
        process_kill_zero_probe_parenthesized_receiver_freeze_bracket_source(),
    );
    let expected_test = format!(
        "const zero = 0; const zeroAlias = zero; {} {} {} {} Kali.test('process kill alias', () => {{ if ({}) {{ throw new Error('expected zero probe'); }} }});\n",
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
fn test_object_has_own_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = object_has_own_frozen_callable_aliases();
    let source = object_has_own_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        r#"Object.freeze(Object.hasOwn)"#,
        r#"Object.freeze((Object.hasOwn))"#,
        r#"Object.freeze(Object["hasOwn"])"#,
        r#"Object.freeze((Object["hasOwn"]))"#,
        r#"Object.freeze(globalThis.Object.hasOwn)"#,
        r#"Object.freeze((globalThis.Object.hasOwn))"#,
        r#"Object.freeze(globalThis.Object["hasOwn"])"#,
        r#"Object.freeze((globalThis.Object["hasOwn"]))"#,
        r#"Object.freeze(globalThis["Object"].hasOwn)"#,
        r#"Object.freeze((globalThis["Object"].hasOwn))"#,
        r#"Object.freeze(globalThis["Object"]["hasOwn"])"#,
        r#"Object.freeze((globalThis["Object"]["hasOwn"]))"#,
    ] {
        assert!(
            aliases.contains(&expected_alias),
            "missing alias: {expected_alias}"
        );
    }

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Object.hasOwn frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, expected);
}

#[test]
fn test_object_has_own_frozen_callable_condition_source_lists_all_aliases_in_order() {
    let aliases = object_has_own_frozen_callable_aliases();
    let condition_source = object_has_own_frozen_callable_condition_source("wrapped", r#""a""#);
    let expected = aliases
        .iter()
        .map(|alias| format!("!{alias}(wrapped, \"a\")"))
        .collect::<Vec<_>>()
        .join(" || ");

    assert_eq!(condition_source, expected);
}

#[test]
fn test_object_has_own_combined_frozen_callable_condition_source_reuses_both_helpers_once() {
    let source = object_has_own_combined_frozen_callable_condition_source("wrapped", r#""a""#);
    assert_eq!(
        source,
        format!(
            "{} || {}",
            object_has_own_frozen_callable_condition_source("wrapped", r#""a""#),
            object_has_own_property_call_frozen_callable_condition_source("wrapped", r#""a""#)
        )
    );
    assert_eq!(
        source
            .matches(&object_has_own_frozen_callable_condition_source(
                "wrapped", r#""a""#
            ))
            .count(),
        1,
        "source: {source}"
    );
    assert_eq!(
        source
            .matches(
                &object_has_own_property_call_frozen_callable_condition_source("wrapped", r#""a""#)
            )
            .count(),
        1,
        "source: {source}"
    );
}

#[test]
fn test_object_has_own_property_call_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = object_has_own_property_call_frozen_callable_aliases();
    let source = object_has_own_property_call_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        r#"Object.freeze(globalThis.Object.prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis.Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call))"#,
        r#"Object.freeze(globalThis["Object"].prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze((globalThis["Object"].prototype.hasOwnProperty["call"]))"#,
        r#"Object.freeze(globalThis.Object.prototype["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((globalThis.Object.prototype["hasOwnProperty"]["call"]))"#,
        r#"Object.freeze(globalThis["Object"]["prototype"]["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]))"#,
        r#"Object.freeze(Object.prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze(Object.prototype["hasOwnProperty"].call)"#,
        r#"Object.freeze((Object.prototype["hasOwnProperty"].call))"#,
        r#"Object.freeze(Object["prototype"].hasOwnProperty.call)"#,
        r#"Object.freeze((Object["prototype"].hasOwnProperty.call))"#,
        r#"Object.freeze(Object["prototype"]["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((Object["prototype"]["hasOwnProperty"]["call"]))"#,
        r#"Object.freeze(Object.prototype.hasOwnProperty["call"])"#,
        r#"Object.freeze((Object.prototype.hasOwnProperty["call"]))"#,
        r#"Object.freeze(Object["prototype"].hasOwnProperty["call"])"#,
        r#"Object.freeze((Object["prototype"].hasOwnProperty["call"]))"#,
    ] {
        assert!(
            aliases.contains(&expected_alias),
            "missing alias: {expected_alias}"
        );
    }

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Object.prototype.hasOwnProperty.call frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, expected);
}

#[test]
fn test_object_has_own_property_call_frozen_callable_condition_source_lists_all_aliases_in_order() {
    let aliases = object_has_own_property_call_frozen_callable_aliases();
    let condition_source =
        object_has_own_property_call_frozen_callable_condition_source("wrapped", r#""a""#);
    let expected = aliases
        .iter()
        .map(|alias| format!("!{alias}(wrapped, \"a\")"))
        .collect::<Vec<_>>()
        .join(" || ");

    assert_eq!(condition_source, expected);
}

#[test]
fn test_object_has_own_property_call_binding_source_is_canonical() {
    let binding_source = object_has_own_property_call_binding_source("hasOwnPropertyCall");

    assert_eq!(
        binding_source,
        "const hasOwnPropertyCall = Object.prototype.hasOwnProperty.call;"
    );
}

#[test]
fn test_late_compat_object_has_own_source_lists_representative_aliases_in_order() {
    let source = late_compat_object_has_own_source("globalThis", r#""a""#);

    for expected in [
        r#"Object.hasOwn(globalThis, "a")"#,
        r#"globalThis.Object.hasOwn(globalThis, "a")"#,
        r#"Object.prototype.hasOwnProperty.call(globalThis, "a")"#,
        r#"Object.prototype.hasOwnProperty["call"](globalThis, "a")"#,
        r#"Object["prototype"].hasOwnProperty.call(globalThis, "a")"#,
        r#"Object["prototype"].hasOwnProperty["call"](globalThis, "a")"#,
        r#"Object["prototype"]["hasOwnProperty"]["call"](globalThis, "a")"#,
        r#"Object.prototype["hasOwnProperty"].call(globalThis, "a")"#,
        r#"globalThis.Object["prototype"].hasOwnProperty["call"](globalThis, "a")"#,
        r#"globalThis['Object'].hasOwnProperty.call(globalThis, "a")"#,
        r#"globalThis['Object']['prototype']['hasOwnProperty']['call'](globalThis, "a")"#,
        r#"globalThis['Object']["prototype"]["hasOwnProperty"]["call"](globalThis, "a")"#,
        r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](globalThis, "a")"#,
    ] {
        assert!(source.contains(expected), "missing alias: {expected}");
    }

    assert!(
        source.ends_with(';'),
        "source should be semicolon-terminated: {source}"
    );
}

#[test]
fn test_number_predicates_source_helpers_are_canonical() {
    assert_eq!(
        number_predicates_preamble_source("1"),
        "const alias = 1; const finite = Number.isFinite; const integer = Number.isInteger; const safeInteger = Number.isSafeInteger;"
    );
    assert_eq!(
        number_predicates_preamble_source("1 as const"),
        "const alias = 1 as const; const finite = Number.isFinite; const integer = Number.isInteger; const safeInteger = Number.isSafeInteger;"
    );
    assert_eq!(
        number_predicates_console_log_body_source(),
        concat!(
            "console.log(Number.isFinite(alias)); ",
            "console.log(integer(alias)); ",
            "console.log(Number.isSafeInteger(alias)); ",
            "console.log(integer(1.5)); ",
            "console.log(Number.isFinite(\"hello\")); ",
            "console.log(Number.isSafeInteger(1.5)); ",
            "console.log(globalThis[\"Number\"][\"isNaN\"](NaN)); ",
            "console.log(globalThis.Number.isNaN(1)); ",
            "console.log(globalThis[\"Number\"].isNaN(1)); ",
            "console.log(globalThis[\"Number\"][\"isFinite\"](alias)); ",
            "console.log(globalThis[\"Number\"][\"isInteger\"](alias)); ",
            "console.log(globalThis[\"Number\"][\"isSafeInteger\"](alias)); ",
            "console.log(globalThis.Number[\"isNaN\"](1)); ",
            "console.log(globalThis[\"Number\"].isFinite(alias)); ",
            "console.log(globalThis.Number[\"isInteger\"](alias)); ",
            "console.log(globalThis[\"Number\"].isSafeInteger(alias)); ",
            "console.log(Number[\"isFinite\"](alias)); ",
            "console.log(Number[\"isInteger\"](alias)); ",
            "console.log(Number[\"isSafeInteger\"](alias)); ",
            "console.log(Number[\"isNaN\"](1)); ",
            "console.log(finite(alias)); ",
            "console.log(integer(alias)); ",
            "console.log(safeInteger(alias));"
        )
    );
    assert_eq!(
        number_predicates_runtime_source(),
        format!(
            "{} {}",
            number_predicates_preamble_source("1"),
            number_predicates_console_log_body_source()
        )
    );
    assert_eq!(
        number_predicates_test_source(),
        format!(
            "Kali.test('number predicates', () => {{ {} {} }});",
            number_predicates_preamble_source("1"),
            number_predicates_console_log_body_source()
        )
    );
    assert!(
        number_predicates_browser_bundle_source("1").starts_with(
            "// kali-tree-shake: browserNumberPredicates\nasync function browserNumberPredicates() {\n  const alias = 1;"
        )
    );
    assert!(
        number_predicates_browser_bundle_source("1 as const").contains("const alias = 1 as const;")
    );
    assert!(number_predicates_browser_bundle_source("1")
        .contains("Number.isSafeInteger(await alias) !== true"));
    assert!(number_predicates_browser_bundle_source("1").ends_with("}\n"));
}

#[test]
fn test_math_floor_trunc_ceil_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = math_floor_trunc_ceil_frozen_callable_aliases();
    let source = math_floor_trunc_ceil_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        r#"Object.freeze(globalThis.Math["floor"])"#,
        r#"Object.freeze((globalThis.Math["floor"]))"#,
        r#"Object.freeze(globalThis.Math.floor)"#,
        r#"Object.freeze((globalThis.Math.floor))"#,
        r#"Object.freeze(globalThis["Math"]["floor"])"#,
        r#"Object.freeze((globalThis["Math"]["floor"]))"#,
        r#"Object.freeze(globalThis["Math"].floor)"#,
        r#"Object.freeze((globalThis["Math"].floor))"#,
        r#"Object.freeze(Math["floor"])"#,
        r#"Object.freeze((Math["floor"]))"#,
        r#"Object.freeze(globalThis.Math["trunc"])"#,
        r#"Object.freeze((globalThis.Math["trunc"]))"#,
        r#"Object.freeze(globalThis.Math.trunc)"#,
        r#"Object.freeze((globalThis.Math.trunc))"#,
        r#"Object.freeze(globalThis["Math"]["trunc"])"#,
        r#"Object.freeze((globalThis["Math"]["trunc"]))"#,
        r#"Object.freeze(globalThis["Math"].trunc)"#,
        r#"Object.freeze((globalThis["Math"].trunc))"#,
        r#"Object.freeze(Math["trunc"])"#,
        r#"Object.freeze((Math["trunc"]))"#,
        r#"Object.freeze(globalThis.Math["ceil"])"#,
        r#"Object.freeze((globalThis.Math["ceil"]))"#,
        r#"Object.freeze(globalThis.Math.ceil)"#,
        r#"Object.freeze((globalThis.Math.ceil))"#,
        r#"Object.freeze(globalThis["Math"]["ceil"])"#,
        r#"Object.freeze((globalThis["Math"]["ceil"]))"#,
        r#"Object.freeze(globalThis["Math"].ceil)"#,
        r#"Object.freeze((globalThis["Math"].ceil))"#,
        r#"Object.freeze(Math["ceil"])"#,
        r#"Object.freeze((Math["ceil"]))"#,
    ] {
        assert!(
            aliases.contains(&expected_alias),
            "missing alias: {expected_alias}"
        );
    }

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Math.floor / Math.trunc / Math.ceil frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, expected);
}

#[test]
fn test_math_pow_source_lists_all_aliases_in_order() {
    let aliases = math_pow_aliases();
    let source = math_pow_source();
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        "Math.pow",
        r#"Math['pow']"#,
        r#"Math["pow"]"#,
        "globalThis.Math.pow",
        r#"globalThis.Math['pow']"#,
        r#"globalThis.Math["pow"]"#,
        r#"globalThis['Math'].pow"#,
        r#"globalThis['Math']['pow']"#,
        r#"globalThis["Math"].pow"#,
        r#"globalThis["Math"]["pow"]"#,
    ] {
        assert!(
            aliases.contains(&expected_alias),
            "missing alias: {expected_alias}"
        );
    }

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Math.pow inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, expected);
}

#[test]
fn test_math_pow_alias_inventory_source_reuses_the_shared_helper_sources_once() {
    let source = math_pow_alias_inventory_source();
    assert_eq!(
        source,
        format!(
            "{} {}",
            math_pow_source().trim_end(),
            math_pow_frozen_callable_source().trim_end()
        )
    );
    assert_eq!(
        source.matches(&math_pow_source()).count(),
        1,
        "source: {source}"
    );
    assert_eq!(
        source.matches(&math_pow_frozen_callable_source()).count(),
        1,
        "source: {source}"
    );
}

#[test]
fn test_math_pow_browser_alias_inventory_aliases_list_all_aliases_in_order() {
    let aliases = math_pow_browser_alias_inventory_aliases();
    let source = math_pow_browser_alias_inventory_source();

    assert_eq!(
        aliases,
        &[
            "Math.pow",
            r#"Math['pow']"#,
            r#"Math["pow"]"#,
            "globalThis.Math.pow",
            r#"globalThis.Math['pow']"#,
            r#"globalThis.Math["pow"]"#,
            r#"globalThis['Math'].pow"#,
            r#"globalThis['Math']['pow']"#,
            r#"globalThis["Math"].pow"#,
            r#"globalThis["Math"]["pow"]"#,
            r#"Object.freeze(globalThis.Math['pow'])"#,
            r#"Object.freeze(globalThis.Math["pow"])"#,
            r#"Object.freeze(globalThis['Math']['pow'])"#,
            r#"Object.freeze(globalThis["Math"]["pow"])"#,
            r#"Object.freeze(globalThis.Math.pow)"#,
            r#"Object.freeze(globalThis['Math'].pow)"#,
            r#"Object.freeze(globalThis["Math"].pow)"#,
            r#"Object.freeze(Math.pow)"#,
            r#"Object.freeze(Math['pow'])"#,
            r#"Object.freeze(Math["pow"])"#,
            r#"Object.freeze((globalThis.Math['pow']))"#,
            r#"Object.freeze((globalThis.Math["pow"]))"#,
            r#"Object.freeze((globalThis['Math']['pow']))"#,
            r#"Object.freeze((globalThis["Math"]["pow"]))"#,
            r#"Object.freeze((globalThis.Math.pow))"#,
            r#"Object.freeze((globalThis['Math'].pow))"#,
            r#"Object.freeze((globalThis["Math"].pow))"#,
            r#"Object.freeze((Math.pow))"#,
            r#"Object.freeze((Math['pow']))"#,
            r#"Object.freeze((Math["pow"]))"#,
        ]
    );

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Math.pow browser inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(source, format!("{};", aliases.join("; ")));
}

#[test]
fn test_math_pow_browser_alias_inventory_source_is_canonical() {
    let source = math_pow_browser_alias_inventory_source();
    let aliases = math_pow_browser_alias_inventory_aliases();
    assert_eq!(source, format!("{};", aliases.join("; ")));
    assert_eq!(
        source.matches(&math_pow_alias_inventory_source()).count(),
        1,
        "source: {source}"
    );
    assert_eq!(
        source
            .matches(&math_pow_bracketed_frozen_callable_source())
            .count(),
        0,
        "source: {source}"
    );
}

#[test]
fn test_math_pow_browser_alias_inventory_source_reuses_the_canonical_math_pow_alias_inventory() {
    let source = math_pow_browser_alias_inventory_source();
    let canonical = math_pow_alias_inventory_source();

    assert_eq!(source, canonical);
    assert_eq!(source.matches(&canonical).count(), 1, "source: {source}");
}

#[test]
fn test_math_pow_browser_alias_inventory_invocation_source_is_canonical() {
    let source = math_pow_browser_alias_inventory_invocation_source();
    let expected = format!(
        "const exponent = 3; const alias = exponent;\n{}\n",
        math_pow_invocation_lines_for_aliases(
            math_pow_browser_alias_inventory_aliases().as_slice(),
            "2",
            "alias",
            "",
        )
    );

    assert_eq!(source, expected);
}

#[test]
fn test_math_pow_bracketed_global_this_alias_chain_source_is_canonical() {
    assert_eq!(
        math_pow_bracketed_global_this_alias_chain_source(),
        concat!(
            "// kali-tree-shake: bracketedGlobalThisMathPowAliasChain\n",
            "function bracketedGlobalThisMathPowAliasChain() {\n",
            "  const exponent = 3;\n",
            "  const alias = exponent;\n",
            "  console.log(globalThis[\"Math\"].pow(2, alias));\n",
            "  return globalThis[\"Math\"].pow(2, alias);\n",
            "}\n",
        )
    );
}

#[test]
fn test_promise_all_settled_browser_body_source_includes_the_shared_freeze_wrapper_aliases() {
    let body = promise_all_settled_browser_body_source();

    assert!(
        body.contains("const frozenBracketedSettled = await Object.freeze(globalThis[\"Promise\"][\"allSettled\"])([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("const parenthesizedFrozenBracketedSettled = await Object.freeze((globalThis[\"Promise\"][\"allSettled\"]))([Promise.resolve(1), Promise.reject('boom')]);"),
        "body: {body}"
    );
    assert!(
        body.contains("parenthesizedFrozenBracketedSettled.length !== 2"),
        "body: {body}"
    );
    assert!(
        body.contains("throw new Error('unexpected Promise.allSettled semantics');"),
        "body: {body}"
    );
    assert!(body.contains("  }\n"), "body: {body}");
}

#[test]
fn test_math_pow_frozen_callable_source_lists_all_aliases_in_order() {
    let direct_aliases = math_pow_frozen_callable_direct_aliases();
    let parenthesized_aliases = math_pow_frozen_callable_parenthesized_aliases();
    let aliases = math_pow_frozen_callable_aliases();
    let source = math_pow_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        r#"Object.freeze(globalThis.Math['pow'])"#,
        r#"Object.freeze(globalThis.Math["pow"])"#,
        r#"Object.freeze(globalThis['Math']['pow'])"#,
        r#"Object.freeze(globalThis["Math"]["pow"])"#,
        r#"Object.freeze(globalThis.Math.pow)"#,
        r#"Object.freeze(globalThis['Math'].pow)"#,
        r#"Object.freeze(globalThis["Math"].pow)"#,
        r#"Object.freeze(Math.pow)"#,
        r#"Object.freeze(Math['pow'])"#,
        r#"Object.freeze(Math["pow"])"#,
    ] {
        assert!(
            direct_aliases.contains(&expected_alias),
            "missing direct alias: {expected_alias}"
        );
    }

    for expected_alias in [
        r#"Object.freeze((globalThis.Math['pow']))"#,
        r#"Object.freeze((globalThis.Math["pow"]))"#,
        r#"Object.freeze((globalThis['Math']['pow']))"#,
        r#"Object.freeze((globalThis["Math"]["pow"]))"#,
        r#"Object.freeze((globalThis.Math.pow))"#,
        r#"Object.freeze((globalThis['Math'].pow))"#,
        r#"Object.freeze((globalThis["Math"].pow))"#,
        r#"Object.freeze((Math.pow))"#,
        r#"Object.freeze((Math['pow']))"#,
        r#"Object.freeze((Math["pow"]))"#,
    ] {
        assert!(
            parenthesized_aliases.contains(&expected_alias),
            "missing parenthesized alias: {expected_alias}"
        );
    }

    let mut unique_aliases = std::collections::HashSet::new();
    for alias in aliases.iter().copied() {
        assert!(
            unique_aliases.insert(alias),
            "duplicate alias in Math.pow frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
    assert_eq!(
        aliases.len(),
        direct_aliases.len() + parenthesized_aliases.len()
    );
    assert_eq!(source, expected);
}

#[test]
fn test_math_pow_bracketed_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = math_pow_bracketed_frozen_callable_aliases();
    let source = math_pow_bracketed_frozen_callable_source();

    assert_eq!(
        aliases,
        &[
            r#"Object.freeze((globalThis.Math["pow"]))"#,
            r#"Object.freeze((globalThis["Math"]["pow"]))"#,
        ]
    );
    assert_eq!(source, "Object.freeze((globalThis.Math[\"pow\"])); Object.freeze((globalThis[\"Math\"][\"pow\"]));");
}

#[test]
fn test_math_pow_invocation_lines_are_canonical() {
    let source = math_pow_invocation_lines(&math_pow_source(), "  ");
    let direct = math_pow_invocation_lines_for_aliases(math_pow_aliases(), "2", "alias", "  ");
    let direct_entries =
        math_pow_invocation_entries_for_aliases(math_pow_aliases(), "2", "alias", "    ");
    let expected = concat!(
        "  console.log(Math.pow(2, alias));\n",
        "  console.log(Math['pow'](2, alias));\n",
        "  console.log(Math[\"pow\"](2, alias));\n",
        "  console.log(globalThis.Math.pow(2, alias));\n",
        "  console.log(globalThis.Math['pow'](2, alias));\n",
        "  console.log(globalThis.Math[\"pow\"](2, alias));\n",
        "  console.log(globalThis['Math'].pow(2, alias));\n",
        "  console.log(globalThis['Math']['pow'](2, alias));\n",
        "  console.log(globalThis[\"Math\"].pow(2, alias));\n",
        "  console.log(globalThis[\"Math\"][\"pow\"](2, alias));"
    );
    let expected_entries = concat!(
        "    Math.pow(2, alias),\n",
        "    Math['pow'](2, alias),\n",
        "    Math[\"pow\"](2, alias),\n",
        "    globalThis.Math.pow(2, alias),\n",
        "    globalThis.Math['pow'](2, alias),\n",
        "    globalThis.Math[\"pow\"](2, alias),\n",
        "    globalThis['Math'].pow(2, alias),\n",
        "    globalThis['Math']['pow'](2, alias),\n",
        "    globalThis[\"Math\"].pow(2, alias),\n",
        "    globalThis[\"Math\"][\"pow\"](2, alias),"
    );

    assert_eq!(source, expected);
    assert_eq!(direct, expected);
    assert_eq!(direct_entries, expected_entries);
}

#[test]
fn test_set_constructor_aliases_and_frozen_callable_source_are_canonical() {
    let aliases = set_constructor_aliases();
    let frozen_aliases = set_constructor_frozen_callable_aliases();
    let source = set_constructor_source();
    let iteration_source = set_constructor_iteration_source();
    let frozen_source = set_constructor_frozen_callable_source();

    assert_eq!(
        aliases,
        &[
            "Set",
            "globalThis.Set",
            r#"globalThis["Set"]"#,
            r#"globalThis['Set']"#
        ]
    );
    assert_eq!(
        source,
        "Set; globalThis.Set; globalThis[\"Set\"]; globalThis['Set'];"
    );
    assert_eq!(
        frozen_aliases,
        &[
            r#"Object.freeze(Set)"#,
            r#"Object.freeze((Set))"#,
            r#"Object.freeze(globalThis.Set)"#,
            r#"Object.freeze((globalThis.Set))"#,
            r#"Object.freeze(globalThis["Set"])"#,
            r#"Object.freeze((globalThis["Set"]))"#,
            r#"Object.freeze(globalThis['Set'])"#,
            r#"Object.freeze((globalThis['Set']))"#,
        ]
    );
    assert_eq!(
        iteration_source,
        concat!(
            "for (const value of new Set([1, 2, 1])) { console.log(value); } ",
            "for (const value of new Set(Object.freeze([1, 2, 1]))) { console.log(value); } ",
            "for (const value of new globalThis.Set([1, 2, 1])) { console.log(value); } ",
            "for (const value of new globalThis[\"Set\"]([1, 2, 1])) { console.log(value); } ",
            "for (const value of new globalThis['Set'](Object.freeze([1, 2, 1]))) { console.log(value); } ",
            "for (const value of new (Object.freeze((Set)))([1, 2, 1])) { console.log(value); } ",
            "for (const value of new (Object.freeze((globalThis.Set)))([1, 2, 1])) { console.log(value); } ",
            "for (const value of new (Object.freeze((globalThis[\"Set\"])))([1, 2, 1])) { console.log(value); }"
        )
    );
    assert_eq!(
        frozen_source,
        concat!(
            "Object.freeze(Set); Object.freeze((Set)); Object.freeze(globalThis.Set); ",
            "Object.freeze((globalThis.Set)); Object.freeze(globalThis[\"Set\"]); ",
            "Object.freeze((globalThis[\"Set\"])); Object.freeze(globalThis['Set']); ",
            "Object.freeze((globalThis['Set']));"
        )
    );
}

#[test]
fn test_map_constructor_aliases_and_frozen_callable_source_are_canonical() {
    let aliases = map_constructor_aliases();
    let frozen_aliases = map_constructor_frozen_callable_aliases();
    let source = map_constructor_source();
    let iteration_source = map_constructor_iteration_source();
    let frozen_source = map_constructor_frozen_callable_source();

    assert_eq!(
        aliases,
        &[
            "Map",
            "globalThis.Map",
            r#"globalThis["Map"]"#,
            r#"globalThis['Map']"#
        ]
    );
    assert_eq!(
        source,
        "Map; globalThis.Map; globalThis[\"Map\"]; globalThis['Map'];"
    );
    assert_eq!(
        frozen_aliases,
        &[
            r#"Object.freeze(Map)"#,
            r#"Object.freeze((Map))"#,
            r#"Object.freeze(globalThis.Map)"#,
            r#"Object.freeze((globalThis.Map))"#,
            r#"Object.freeze(globalThis["Map"])"#,
            r#"Object.freeze((globalThis["Map"]))"#,
            r#"Object.freeze(globalThis['Map'])"#,
            r#"Object.freeze((globalThis['Map']))"#,
        ]
    );
    assert_eq!(
        iteration_source,
        concat!(
            "for (const entry of new Map([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new Map(Object.freeze([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new globalThis.Map([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new globalThis[\"Map\"]([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new globalThis['Map'](Object.freeze([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (Object.freeze((Map)))([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (Object.freeze((globalThis.Map)))([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); } ",
            "for (const entry of new (Object.freeze((globalThis[\"Map\"])))([[1, 2], [1, 3], [4, 5]])) { console.log(entry[0], entry[1]); }"
        )
    );
    assert_eq!(
        frozen_source,
        concat!(
            "Object.freeze(Map); Object.freeze((Map)); Object.freeze(globalThis.Map); ",
            "Object.freeze((globalThis.Map)); Object.freeze(globalThis[\"Map\"]); ",
            "Object.freeze((globalThis[\"Map\"])); Object.freeze(globalThis['Map']); ",
            "Object.freeze((globalThis['Map']));"
        )
    );
}

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
            r#"globalThis["process"].exit"#,
            r#"globalThis["process"]["exit"]"#,
            "process[\"exit\"]",
            "globalThis.process[\"exit\"]",
        ]
    );
}

#[test]
fn test_late_process_control_exit_source_lists_all_aliases_in_order() {
    let source = late_process_control_exit_source();
    let expected = concat!(
        "process.exit; ",
        "globalThis.process.exit; ",
        "globalThis[\"process\"].exit; ",
        "globalThis[\"process\"][\"exit\"]; ",
        "process[\"exit\"]; ",
        "globalThis.process[\"exit\"];"
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
        prefix.ends_with("globalThis.process[\"exit\"];"),
        "prefix should preserve the process-control preamble: {prefix}"
    );
}

#[test]
fn test_late_process_control_single_quoted_process_source_reuses_the_shared_zero_probe_inventory_once(
) {
    let source = late_process_control_single_quoted_process_source();
    let zero_probe_source = late_process_control_source();
    let single_quoted_process_source =
        join_semicolon_terminated_segments(LATE_PROCESS_CONTROL_SINGLE_QUOTED_PROCESS_SEGMENTS);
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

#[test]
fn test_late_process_env_mutation_source_lists_bracketed_process_aliases_and_mixed_delete_aliases()
{
    let aliases = late_process_env_mutation_aliases();
    let source = late_process_env_mutation_source();
    let expected = format!("{};", aliases.join("; "));

    assert_eq!(
        aliases,
        &[
            r#"process.env = {}"#,
            r#"process.env.KALI_BROWSER_ENV_MUTATION = {}"#,
            r#"globalThis.process.env = {}"#,
            r#"globalThis.process.env.KALI_BROWSER_ENV_MUTATION = {}"#,
            r#"process["env"] = {}"#,
            r#"process["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
            r#"process["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
            r#"globalThis.process["env"] = {}"#,
            r#"globalThis.process["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
            r#"globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
            r#"delete globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
            r#"globalThis["process"].env = {}"#,
            r#"globalThis["process"].env.KALI_BROWSER_ENV_MUTATION = {}"#,
            r#"globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"] = {}"#,
            r#"globalThis["process"]["env"] = {}"#,
            r#"globalThis["process"]["env"].KALI_BROWSER_ENV_MUTATION = {}"#,
            r#"globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"] = {}"#,
            r#"delete process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
            r#"delete process.env["KALI_BROWSER_ENV_MUTATION"]"#,
            r#"delete globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
            r#"delete globalThis.process.env["KALI_BROWSER_ENV_MUTATION"]"#,
            r#"delete globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"]"#,
            r#"delete globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
        ]
    );
    assert_eq!(source, expected);
    assert!(
        source.contains(r#"globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"] = {}"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"delete globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"delete globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"delete globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"delete globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#),
        "source: {source}"
    );
}
