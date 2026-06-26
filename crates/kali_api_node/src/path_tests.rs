use crate::*;
use std::path::PathBuf;

#[test]
fn path_helpers_are_lexical_and_deterministic() {
    assert_eq!(
        normalize_path("./foo/../bar//baz"),
        PathBuf::from("bar/baz")
    );
    assert_eq!(
        join_path("/tmp", "project/src"),
        PathBuf::from("/tmp/project/src")
    );
    assert_eq!(
        resolve_path("/tmp/project", "../lib/index.js"),
        PathBuf::from("/tmp/lib/index.js")
    );
    assert_eq!(
        relative_path("/tmp/project/src", "/tmp/project/lib/index.js"),
        PathBuf::from("../lib/index.js")
    );
    assert_eq!(
        dirname("/tmp/project/src/main.ts"),
        PathBuf::from("/tmp/project/src")
    );
    assert_eq!(basename("/tmp/project/src/main.ts"), "main.ts");
    assert_eq!(extname("/tmp/project/src/main.ts"), ".ts");
}
