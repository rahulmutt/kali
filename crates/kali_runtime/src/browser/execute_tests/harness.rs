use super::*;

#[test]
fn browser_harness_invocation_checked_builds_a_launch_plan() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let script = tempdir.path().join("browser-harness.mjs");
    let args = vec!["alpha".to_string(), "beta".to_string()];

    let invocation = browser_harness_invocation_checked(
        Some("node --experimental-fetch"),
        &script,
        &args,
        tempdir.path(),
    )
    .expect("build browser harness invocation");

    assert_eq!(invocation.executable, "node");
    assert_eq!(
        invocation.harness_args,
        vec!["--experimental-fetch".to_string()]
    );
    assert_eq!(invocation.script, script);
    assert_eq!(invocation.args, args);
    assert_eq!(invocation.current_dir, tempdir.path());
    assert_eq!(
        invocation.command,
        vec![
            "node".to_string(),
            "--experimental-fetch".to_string(),
            script.display().to_string(),
            "alpha".to_string(),
            "beta".to_string(),
        ]
    );
}

#[cfg(unix)]
#[test]
fn browser_harness_invocation_checked_uses_file_url_for_browser_executables() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let script = tempdir.path().join("browser-harness.html");
    let args = vec!["alpha".to_string(), "beta".to_string()];

    let invocation = browser_harness_invocation_checked(
        Some("chromium --headless"),
        &script,
        &args,
        tempdir.path(),
    )
    .expect("build browser harness invocation");

    assert_eq!(invocation.executable, "chromium");
    assert_eq!(invocation.harness_args, vec!["--headless".to_string()]);
    assert_eq!(invocation.script, script);
    assert_eq!(invocation.args, args);
    assert_eq!(invocation.current_dir, tempdir.path());
    assert!(
        invocation.command[2].starts_with("file://"),
        "command: {:?}",
        invocation.command
    );
    assert!(
        invocation.command[2].contains("browser-harness.html"),
        "command: {:?}",
        invocation.command
    );
    assert_eq!(
        invocation.command,
        vec![
            invocation.executable.clone(),
            "--headless".to_string(),
            invocation.command[2].clone(),
            "alpha".to_string(),
            "beta".to_string(),
        ]
    );
}

#[test]
fn browser_harness_run_checked_launches_command_and_captures_output() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let script = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-harness.mjs",
        r#"
console.error('browser-harness-stderr');
console.log(JSON.stringify(process.argv.slice(2)));
process.exit(7);
"#,
    );

    let outcome = browser_harness_run_checked(
        Some("node"),
        &script,
        &["alpha".to_string(), "beta".to_string()],
        tempdir.path(),
    )
    .expect("launch browser harness");

    assert_eq!(
        outcome.command,
        vec![
            "node".to_string(),
            script.display().to_string(),
            "alpha".to_string(),
            "beta".to_string(),
        ]
    );
    assert_eq!(outcome.status.code(), Some(7));
    assert!(
        outcome.stdout.contains(r#"["alpha","beta"]"#),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome.stderr.contains("browser-harness-stderr"),
        "stderr: {}",
        outcome.stderr
    );
}

#[test]
fn browser_harness_launch_failure_preserves_the_resolved_command_vector() {
    let tempdir = kali_test_support::fixtures::tempdir();
    let script = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-harness.mjs",
        "console.log('unreachable');",
    );

    let error = browser_harness_run_checked(
        Some("definitely-not-a-real-browser-runner"),
        &script,
        &["alpha".to_string(), "beta".to_string()],
        tempdir.path(),
    )
    .expect_err("launch should fail for a missing executable");

    match error {
        BrowserHarnessError::LaunchFailed {
            executable,
            script: error_script,
            command,
            message,
        } => {
            assert_eq!(executable, "definitely-not-a-real-browser-runner");
            assert_eq!(error_script, script);
            assert_eq!(
                command,
                vec![
                    "definitely-not-a-real-browser-runner".to_string(),
                    script.display().to_string(),
                    "alpha".to_string(),
                    "beta".to_string(),
                ]
            );
            assert!(
                message.contains("No such file") || message.contains("not found"),
                "message: {message}"
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
