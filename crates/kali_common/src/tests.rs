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
fn test_process_kill_zero_probe_source_lists_all_aliases_in_order() {
    let direct = process_kill_zero_probe_direct_zero_aliases();
    let wrapped = process_kill_zero_probe_wrapped_zero_aliases();
    let aliases = process_kill_zero_probe_aliases();
    let source = process_kill_zero_probe_source();
    let inventory_source = process_kill_zero_probe_alias_inventory_source();
    let direct_source = process_kill_zero_probe_direct_source();
    let wrapped_source = process_kill_zero_probe_wrapped_source();
    let expected = format!("{}", aliases.join("; ")) + ";";

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
fn test_process_kill_zero_probe_typed_wrapper_sources_list_all_call_targets_in_order() {
    let targets = process_kill_zero_probe_call_target_aliases();
    let satisfies_source = process_kill_zero_probe_satisfies_source();
    let type_assertion_source = process_kill_zero_probe_type_assertion_source();
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
    assert_eq!(satisfies_source, expected_satisfies);
    assert_eq!(type_assertion_source, expected_type_assertion);
}

#[test]
fn test_object_has_own_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = object_has_own_frozen_callable_aliases();
    let source = object_has_own_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        r#"Object.freeze(globalThis.Object["hasOwn"])"#,
        r#"Object.freeze((globalThis.Object["hasOwn"]))"#,
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
fn test_object_has_own_property_call_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = object_has_own_property_call_frozen_callable_aliases();
    let source = object_has_own_property_call_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        r#"Object.freeze(globalThis.Object.prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis.Object.prototype.hasOwnProperty.call))"#,
        r#"Object.freeze(globalThis["Object"].prototype.hasOwnProperty.call)"#,
        r#"Object.freeze((globalThis["Object"].prototype.hasOwnProperty.call))"#,
        r#"Object.freeze(globalThis.Object.prototype["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((globalThis.Object.prototype["hasOwnProperty"]["call"]))"#,
        r#"Object.freeze(globalThis["Object"]["prototype"]["hasOwnProperty"]["call"])"#,
        r#"Object.freeze((globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]))"#,
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
fn test_math_floor_trunc_ceil_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = math_floor_trunc_ceil_frozen_callable_aliases();
    let source = math_floor_trunc_ceil_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        r#"Object.freeze(globalThis.Math["floor"])"#,
        r#"Object.freeze((globalThis.Math["floor"]))"#,
        r#"Object.freeze(globalThis["Math"]["floor"])"#,
        r#"Object.freeze((globalThis["Math"]["floor"]))"#,
        r#"Object.freeze(globalThis.Math["trunc"])"#,
        r#"Object.freeze((globalThis.Math["trunc"]))"#,
        r#"Object.freeze(globalThis["Math"]["trunc"])"#,
        r#"Object.freeze((globalThis["Math"]["trunc"]))"#,
        r#"Object.freeze(globalThis.Math["ceil"])"#,
        r#"Object.freeze((globalThis.Math["ceil"]))"#,
        r#"Object.freeze(globalThis["Math"]["ceil"])"#,
        r#"Object.freeze((globalThis["Math"]["ceil"]))"#,
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
fn test_math_pow_frozen_callable_source_lists_all_aliases_in_order() {
    let aliases = math_pow_frozen_callable_aliases();
    let source = math_pow_frozen_callable_source();
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        r#"Object.freeze(globalThis.Math["pow"])"#,
        r#"Object.freeze((globalThis.Math["pow"]))"#,
        r#"Object.freeze(globalThis["Math"]["pow"])"#,
        r#"Object.freeze((globalThis["Math"]["pow"]))"#,
        r#"Object.freeze(globalThis.Math.pow)"#,
        r#"Object.freeze((globalThis.Math.pow))"#,
        r#"Object.freeze(globalThis["Math"].pow)"#,
        r#"Object.freeze((globalThis["Math"].pow))"#,
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
            "duplicate alias in Math.pow frozen-callable inventory: {alias}"
        );
    }

    assert_eq!(aliases.len(), unique_aliases.len());
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
        source.contains(r#"globalThis["process"]["exit"]"#),
        "source: {source}"
    );
    assert!(
        prefix.ends_with("globalThis.process[\"exit\"];"),
        "prefix should preserve the process-control preamble: {prefix}"
    );
}

#[test]
fn test_late_process_env_mutation_source_lists_bracketed_process_aliases() {
    let source = late_process_env_mutation_source();
    assert!(
        source.contains(r#"process["env"]["KALI_BROWSER_ENV_MUTATION"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"delete globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"]"#),
        "source: {source}"
    );
}
