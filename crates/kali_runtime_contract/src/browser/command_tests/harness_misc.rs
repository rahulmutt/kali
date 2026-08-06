use super::*;

#[test]
fn browser_harness_recognizes_all_canonical_browser_executable_names() {
    for executable in BROWSER_HARNESS_BROWSER_EXECUTABLE_NAMES {
        let parts = match browser_harness_command_parts_for_browser_executable(executable) {
            Some(parts) => parts,
            None => panic!(
                "recognized browser alias should be treated as a browser executable: {executable}"
            ),
        };
        assert_eq!(
            parts,
            vec![executable.to_string(), "--headless".to_string()]
        );
        assert!(browser_harness_uses_html_entrypoint(executable));
    }
}
