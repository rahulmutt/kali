use crate::*;
use std::path::PathBuf;

#[test]
fn os_and_url_helpers_expose_expected_views() {
    let os = NodeOs;
    assert!(!os.platform().is_empty());
    assert!(!os.arch().is_empty());
    assert!(matches!(os.eol(), "\n" | "\r\n"));
    assert!(os.cpus() >= 1);
    assert_eq!(os.tmpdir(), std::env::temp_dir());

    let expected_home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from));
    assert_eq!(os.home_dir(), expected_home);

    let parsed = NodeUrl::parse("https://example.com/path?query=1").expect("url");
    assert_eq!(parsed.as_str(), "https://example.com/path?query=1");

    let resolved = NodeUrl::resolve("https://example.com/base/", "../child").expect("resolve");
    assert_eq!(resolved.as_str(), "https://example.com/child");
}
