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
    assert_eq!(
        generator_function_lowering_unavailable_message_for_flavors(true, true),
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
    let expected = format!("{};", aliases.join("; "));

    for expected_alias in [
        r#"process.kill"#,
        r#"globalThis.process.kill"#,
        r#"process["kill"]((0))"#,
        r#"globalThis["process"]["kill"]((0))"#,
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
