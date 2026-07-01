use super::*;

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
        matches!(default_parts[0].as_str(), "node" | "bun" | "deno")
            || browser_harness_uses_html_entrypoint(&default_parts[0]),
        "default browser harness command should prefer a JavaScript runtime, falling back to a browser executable"
    );
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

#[test]
fn browser_harness_default_command_prefers_js_runtime_over_browser() {
    // A JS runtime is chosen ahead of any installed browser, in node/bun/deno order.
    assert_eq!(
        browser_harness_default_command_parts_from(|exe| matches!(exe, "node" | "chromium")),
        vec!["node".to_string()]
    );
    assert_eq!(
        browser_harness_default_command_parts_from(|exe| matches!(
            exe,
            "bun" | "deno" | "chromium"
        )),
        vec!["bun".to_string()]
    );
    assert_eq!(
        browser_harness_default_command_parts_from(|exe| matches!(exe, "deno" | "chromium")),
        vec!["deno".to_string()]
    );

    // A real browser is used only when no JS runtime is available.
    assert_eq!(
        browser_harness_default_command_parts_from(|exe| exe == "chromium"),
        vec!["chromium".to_string(), "--headless".to_string()]
    );

    // With nothing available, fall back to node for a stable error surface.
    assert_eq!(
        browser_harness_default_command_parts_from(|_| false),
        vec!["node".to_string()]
    );
}
