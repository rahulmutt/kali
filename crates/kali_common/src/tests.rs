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
