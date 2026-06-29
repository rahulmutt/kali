use super::*;

#[test]
fn browser_harness_invocation_checked_preserves_html_entrypoint_file_urls_for_paths_with_spaces() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let script = tempdir.path().join("browser app.html");
    let current_dir = tempdir.path().join("browser cwd");
    std::fs::create_dir_all(&current_dir).expect("create browser cwd");
    let args = vec!["one".to_string(), "two words".to_string()];

    let invocation = browser_harness_invocation_checked(
        Some(r#"chrome --headless --profile "real browser""#),
        &script,
        &args,
        &current_dir,
    )
    .expect("build browser harness invocation");

    assert_eq!(invocation.executable, "chrome");
    assert_eq!(
        invocation.harness_args,
        vec![
            "--headless".to_string(),
            "--profile".to_string(),
            "real browser".to_string(),
        ]
    );
    assert_eq!(invocation.script, script);
    assert_eq!(invocation.args, args);
    assert_eq!(invocation.current_dir, current_dir);
    assert_eq!(invocation.command[0], "chrome");
    assert_eq!(invocation.command[1], "--headless");
    assert_eq!(invocation.command[2], "--profile");
    assert_eq!(invocation.command[3], "real browser");
    assert!(
        invocation.command[4].starts_with("file:"),
        "command: {:?}",
        invocation.command
    );
    assert!(
        invocation.command[4].contains("browser%20app.html"),
        "command: {:?}",
        invocation.command
    );
    assert_eq!(invocation.command[5], "one");
    assert_eq!(invocation.command[6], "two words");
}

#[test]
fn browser_harness_launch_failure_reports_the_resolved_command_and_script() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let script = tempdir.path().join("browser app.html");
    let current_dir = tempdir.path().join("browser cwd");
    std::fs::create_dir_all(&current_dir).expect("create browser cwd");
    let args = vec!["one".to_string(), "two words".to_string()];

    let invocation = browser_harness_invocation_checked(
        Some(r#"definitely-not-a-real-browser --headless --profile "real browser""#),
        &script,
        &args,
        &current_dir,
    )
    .expect("build browser harness invocation");
    let expected_command = invocation.command.clone();

    let error = invocation.launch().expect_err("launch should fail");
    let message = error.to_string();

    match error {
        BrowserHarnessError::LaunchFailed {
            executable,
            script: observed_script,
            command,
            message: launch_message,
        } => {
            assert_eq!(executable, "definitely-not-a-real-browser");
            assert_eq!(observed_script, script);
            assert_eq!(command, expected_command);
            assert!(!launch_message.is_empty());
        }
        other => panic!("unexpected browser harness error: {other:?}"),
    }

    assert!(message.contains("failed to launch browser harness command"));
    assert!(message.contains("browser app.html"));
    assert!(message.contains("definitely-not-a-real-browser"));
}

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
