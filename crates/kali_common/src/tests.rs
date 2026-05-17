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
fn test_process_kill_zero_probe_unavailable_message_lists_wrapped_zero_aliases() {
    let message = process_kill_zero_probe_unavailable_message();
    for alias in process_kill_zero_probe_wrapped_zero_aliases() {
        assert!(
            message.contains(alias),
            "missing alias from zero-probe message: {alias}"
        );
    }

    for alias in [
        r#"process.kill(0)"#,
        r#"process.kill(+0)"#,
        r#"process["kill"](0)"#,
        r#"process["kill"](+0)"#,
        r#"process["kill"]((0))"#,
        r#"globalThis.process.kill(0)"#,
        r#"globalThis.process.kill(+0)"#,
        r#"globalThis.process.kill((0))"#,
        r#"globalThis["process"].kill((0))"#,
        r#"globalThis.process["kill"](0)"#,
        r#"globalThis.process["kill"](+0)"#,
        r#"globalThis.process["kill"]((0))"#,
        r#"globalThis["process"].kill(0)"#,
        r#"globalThis["process"].kill(+0)"#,
        r#"globalThis["process"].kill((0))"#,
        r#"globalThis["process"]["kill"](0)"#,
        r#"globalThis["process"]["kill"](+0)"#,
        r#"globalThis["process"]["kill"]((0))"#,
        r#"Object.freeze(process.kill)(0)"#,
        r#"Object.freeze(process.kill)(+0)"#,
        r#"Object.freeze((process.kill))(0)"#,
        r#"Object.freeze((process.kill))(+0)"#,
        r#"Object.freeze(globalThis.process.kill)(0)"#,
        r#"Object.freeze(globalThis.process.kill)(+0)"#,
        r#"Object.freeze(process)["kill"](0)"#,
        r#"Object.freeze(process)["kill"](+0)"#,
        r#"Object.freeze(globalThis.process)["kill"](0)"#,
        r#"Object.freeze(globalThis.process)["kill"](+0)"#,
        r#"Object.freeze(globalThis["process"])["kill"](0)"#,
        r#"Object.freeze(globalThis["process"])["kill"](+0)"#,
        r#"Object.freeze(globalThis.process["kill"])(0)"#,
        r#"Object.freeze(globalThis.process["kill"])(+0)"#,
        r#"Object.freeze((globalThis["process"]["kill"]))(0)"#,
        r#"Object.freeze((globalThis["process"]["kill"]))(+0)"#,
        r#"Object.freeze((globalThis.process["kill"]))(0)"#,
        r#"Object.freeze((globalThis.process["kill"]))(+0)"#,
        r#"Object.freeze(globalThis["process"]["kill"])(0)"#,
        r#"Object.freeze(globalThis["process"]["kill"])(+0)"#,
        r#"Object.freeze(globalThis["process"].kill)(0)"#,
        r#"Object.freeze(globalThis["process"].kill)(+0)"#,
        r#"Object.freeze((globalThis["process"].kill))(0)"#,
        r#"Object.freeze((globalThis["process"].kill))(+0)"#,
        r#"((process.kill))(0)"#,
        r#"((process.kill))(+0)"#,
        r#"((process["kill"]))(0)"#,
        r#"((process["kill"]))(+0)"#,
        r#"((globalThis.process.kill))(0)"#,
        r#"((globalThis.process.kill))(+0)"#,
        r#"((globalThis.process["kill"]))(0)"#,
        r#"((globalThis.process["kill"]))(+0)"#,
        r#"((globalThis["process"].kill))(0)"#,
        r#"((globalThis["process"].kill))(+0)"#,
        r#"((globalThis["process"]["kill"]))(0)"#,
        r#"((globalThis["process"]["kill"]))(+0)"#,
    ] {
        assert!(
            message.contains(alias),
            "missing alias from zero-probe message: {alias}"
        );
    }
}
