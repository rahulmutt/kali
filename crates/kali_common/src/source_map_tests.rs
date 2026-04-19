use super::*;

#[test]
fn test_source_map_creation() {
    let sm = SourceMap::new();
    assert!(sm.registry.get_file(FileId::new(0)).is_none());
}

#[test]
fn test_intern_path() {
    let mut sm = SourceMap::new();
    let path = Path::new("/test/file.ts");
    let fid = sm.intern_path(path);

    assert_eq!(sm.format_file_ref(fid), "file.ts");
}
