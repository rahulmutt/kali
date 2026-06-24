use crate::*;


#[test]
fn split_command_spec_supports_shell_like_quoting() {
    let parts = split_command_spec(
        r#"browser-wrapper --headless --profile "real browser" 'wrapped runner' escaped\ space"#,
    )
    .expect("split valid browser harness command");

    assert_eq!(
        parts,
        vec![
            "browser-wrapper".to_string(),
            "--headless".to_string(),
            "--profile".to_string(),
            "real browser".to_string(),
            "wrapped runner".to_string(),
            "escaped space".to_string(),
        ]
    );
}


#[test]
fn browser_harness_command_parts_exposes_override_and_default_selection() {
    let override_parts = browser_harness_command_parts_for(Some(
        r#"browser-wrapper --headless --profile "real browser" 'wrapped runner' escaped\ space"#,
    ));
    assert_eq!(
        override_parts,
        vec![
            "browser-wrapper".to_string(),
            "--headless".to_string(),
            "--profile".to_string(),
            "real browser".to_string(),
            "wrapped runner".to_string(),
            "escaped space".to_string(),
        ]
    );

    let default_parts = browser_harness_command_parts();
    assert!(
        !default_parts.is_empty(),
        "default browser harness command should not be empty"
    );
    assert!(
        matches!(default_parts[0].as_str(), "bun" | "node")
            || browser_harness_uses_html_entrypoint(&default_parts[0]),
        "default browser harness command should prefer a browser executable when one is available"
    );
}


#[test]
fn browser_harness_invocation_checked_preserves_html_entrypoint_file_urls_for_paths_with_spaces() {
    let tempdir = tempfile::tempdir().expect("tempdir");
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
    let tempdir = tempfile::tempdir().expect("tempdir");
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


#[test]
fn split_command_spec_rejects_malformed_inputs() {
    assert_eq!(split_command_spec("   "), None);
    assert_eq!(split_command_spec(r#"" --flag"#), None);
    assert_eq!(split_command_spec(r#"browser-wrapper "unterminated"#), None);
    assert_eq!(split_command_spec(r#"browser-wrapper \"#), None);
}


#[test]
fn browser_harness_command_parts_checked_reports_malformed_overrides() {
    let empty_override = browser_harness_command_parts_checked(Some(""))
        .expect_err("empty override should be rejected");
    assert!(empty_override.contains("KALI_BROWSER_BUNDLE_HARNESS_COMMAND"));

    let empty_executable = browser_harness_command_parts_checked(Some(r#"" --flag"#))
        .expect_err("empty executable token should be rejected");
    assert!(empty_executable.contains("KALI_BROWSER_BUNDLE_HARNESS_COMMAND"));
    assert!(empty_executable.contains(r#"" --flag"#));

    let flag_only = browser_harness_command_parts_checked(Some("--headless"))
        .expect_err("flag-only command should be rejected");
    assert!(flag_only.contains("KALI_BROWSER_BUNDLE_HARNESS_COMMAND"));
    assert!(flag_only.contains("--headless"));

    let padded_flag_only = browser_harness_command_parts_checked(Some("  --headless  "))
        .expect_err("padded flag-only command should be rejected");
    assert!(padded_flag_only.contains("KALI_BROWSER_BUNDLE_HARNESS_COMMAND"));
    assert!(padded_flag_only.contains("  --headless  "));

    let unterminated =
        browser_harness_command_parts_checked(Some(r#"browser-wrapper "unterminated"#))
            .expect_err("unterminated quotes should be rejected");
    assert!(unterminated.contains("KALI_BROWSER_BUNDLE_HARNESS_COMMAND"));
    assert!(unterminated.contains("browser-wrapper"));
    assert!(unterminated.contains("unterminated"));
}


#[test]
fn browser_harness_command_parts_checked_trims_surrounding_whitespace() {
    let parts = browser_harness_command_parts_checked(Some("\n  node --test --reporter tap  \t"))
        .expect("trimmed browser harness command should parse");

    assert_eq!(parts, vec!["node", "--test", "--reporter", "tap"]);
}


#[test]
fn browser_harness_command_parts_checked_trims_surrounding_whitespace_and_preserves_quotes() {
    let parts = browser_harness_command_parts_checked(Some(
        "\n  chrome --headless --profile \"real browser\"  \t",
    ))
    .expect("trimmed quoted browser harness command should parse");

    assert_eq!(
        parts,
        vec!["chrome", "--headless", "--profile", "real browser"]
    );
}


#[test]
fn browser_harness_command_parts_for_browser_executables_use_headless_mode() {
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("chrome"),
        Some(vec!["chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("chrome-headless-shell"),
        Some(vec![
            "chrome-headless-shell".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("chromium"),
        Some(vec!["chromium".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("chromium-for-testing"),
        Some(vec![
            "chromium-for-testing".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("chromium for testing"),
        Some(vec![
            "chromium for testing".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("/usr/bin/google-chrome-stable"),
        Some(vec![
            "google-chrome-stable".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("Google Chrome.app"),
        Some(vec!["google chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("Google Chrome.command"),
        Some(vec!["google chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("Google Chrome.lnk"),
        Some(vec!["google chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("Google Chrome.lnk.exe"),
        Some(vec!["google chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("Google Chrome.url"),
        Some(vec!["google chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google-chrome.url.exe"),
        Some(vec!["google-chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("brave-browser"),
        Some(vec!["brave-browser".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("brave-browser-stable"),
        Some(vec![
            "brave-browser-stable".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("brave browser stable"),
        Some(vec![
            "brave browser stable".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("opera"),
        Some(vec!["opera".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("vivaldi"),
        Some(vec!["vivaldi".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google chrome stable"),
        Some(vec![
            "google chrome stable".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google chrome beta"),
        Some(vec![
            "google chrome beta".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google chrome canary"),
        Some(vec![
            "google chrome canary".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google chrome dev"),
        Some(vec![
            "google chrome dev".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google chrome for testing"),
        Some(vec![
            "google chrome for testing".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google chrome unstable"),
        Some(vec![
            "google chrome unstable".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google-chrome-dev"),
        Some(vec![
            "google-chrome-dev".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("chromium-dev"),
        Some(vec!["chromium-dev".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("brave-browser-nightly"),
        Some(vec![
            "brave-browser-nightly".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("vivaldi-snapshot"),
        Some(vec![
            "vivaldi-snapshot".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("firefox-beta"),
        Some(vec!["firefox-beta".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("firefox-developer-edition"),
        Some(vec![
            "firefox-developer-edition".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("google-chrome-headless-shell"),
        Some(vec![
            "google-chrome-headless-shell".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("msedge-beta"),
        Some(vec!["msedge-beta".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("msedge-canary"),
        Some(vec!["msedge-canary".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("msedge-dev"),
        Some(vec!["msedge-dev".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("msedge-insider"),
        Some(vec!["msedge-insider".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("msedge-stable"),
        Some(vec!["msedge-stable".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("edge-beta"),
        Some(vec!["edge-beta".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("edge-stable"),
        Some(vec!["edge-stable".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("microsoft-edge-stable"),
        Some(vec![
            "microsoft-edge-stable".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("microsoft edge stable"),
        Some(vec![
            "microsoft edge stable".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("edge-canary"),
        Some(vec!["edge-canary".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("edge-dev"),
        Some(vec!["edge-dev".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("edge-insider"),
        Some(vec!["edge-insider".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("microsoft-edge-canary"),
        Some(vec![
            "microsoft-edge-canary".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("microsoft-edge-insider"),
        Some(vec![
            "microsoft-edge-insider".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable(
            "C:/Program Files/Microsoft/Edge/Application/msedge.exe"
        ),
        Some(vec!["msedge".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable(
            "C:/Program Files/Google/Chrome/Application/chrome.cmd"
        ),
        Some(vec!["chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable(
            "C:/Program Files/Google/Chrome/Application/chrome.ps1"
        ),
        Some(vec!["chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable(
            "C:/Program Files/Google/Chrome/Application/chrome-for-testing.com"
        ),
        Some(vec![
            "chrome-for-testing".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable(
            "C:/Program Files/Google/Chrome/Application/google-chrome-for-testing.com"
        ),
        Some(vec![
            "google-chrome-for-testing".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable(
            "C:/Program Files/Google/Chrome/Application/google-chrome-canary.exe"
        ),
        Some(vec![
            "google-chrome-canary".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable(
            "/usr/share/applications/google-chrome.desktop"
        ),
        Some(vec!["google-chrome".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable(
            "/opt/Thorium Browser/thorium-browser.com"
        ),
        Some(vec![
            "thorium-browser".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("librewolf"),
        Some(vec!["librewolf".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("waterfox"),
        Some(vec!["waterfox".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("mullvad-browser"),
        Some(vec![
            "mullvad-browser".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("mullvad browser"),
        Some(vec![
            "mullvad browser".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("zen-browser"),
        Some(vec!["zen-browser".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("zen browser"),
        Some(vec!["zen browser".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("thorium-browser"),
        Some(vec![
            "thorium-browser".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("thorium browser"),
        Some(vec![
            "thorium browser".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("firefox"),
        Some(vec!["firefox".to_string(), "--headless".to_string()])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("firefox-nightly"),
        Some(vec![
            "firefox-nightly".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("firefox-developer-edition"),
        Some(vec![
            "firefox-developer-edition".to_string(),
            "--headless".to_string()
        ])
    );
    assert_eq!(
        browser_harness_command_parts_for_browser_executable("node"),
        None
    );
}
