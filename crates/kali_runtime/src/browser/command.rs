//! Browser harness command resolution and executable helpers.
use crate::*;

/// Split an argv-style command specification into deterministic tokens.
///
/// The parser accepts the small shell-like subset used by browser harness
/// overrides: whitespace separates tokens, single and double quotes group
/// whitespace, and backslashes escape the next character outside single quotes.
/// The function returns `None` for malformed input such as unterminated quotes,
/// a dangling escape, or an empty or whitespace-only command string.
pub fn split_command_spec(command: &str) -> Option<Vec<String>> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut token_open = false;
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut escaped = false;

    for ch in command.chars() {
        if escaped {
            current.push(ch);
            token_open = true;
            escaped = false;
            continue;
        }

        match ch {
            '\\' if !in_single_quotes => {
                escaped = true;
            }
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
                token_open = true;
            }
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
                token_open = true;
            }
            ch if ch.is_whitespace() && !in_single_quotes && !in_double_quotes => {
                if token_open {
                    parts.push(std::mem::take(&mut current));
                    token_open = false;
                }
            }
            ch => {
                current.push(ch);
                token_open = true;
            }
        }
    }

    if escaped || in_single_quotes || in_double_quotes {
        return None;
    }

    if token_open {
        parts.push(current);
    }

    if parts.is_empty() || parts.first().is_some_and(|part| part.is_empty()) {
        return None;
    }

    Some(parts)
}

pub(crate) fn browser_harness_normalized_executable_name(executable: &str) -> String {
    let executable = Path::new(executable)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(executable)
        .to_ascii_lowercase();

    let mut normalized = executable;
    loop {
        let next = normalized
            .strip_suffix(".desktop")
            .or_else(|| normalized.strip_suffix(".app"))
            .or_else(|| normalized.strip_suffix(".command"))
            .or_else(|| normalized.strip_suffix(".lnk"))
            .or_else(|| normalized.strip_suffix(".exe"))
            .or_else(|| normalized.strip_suffix(".cmd"))
            .or_else(|| normalized.strip_suffix(".bat"))
            .or_else(|| normalized.strip_suffix(".com"))
            .or_else(|| normalized.strip_suffix(".ps1"))
            .or_else(|| normalized.strip_suffix(".url"));
        match next {
            Some(next) => normalized = next.to_string(),
            None => return normalized,
        }
    }
}

pub(crate) const BROWSER_HARNESS_BROWSER_EXECUTABLE_NAMES: &[&str] = &[
    "chrome",
    "chrome-beta",
    "chrome-canary",
    "chrome-headless-shell",
    "chrome-unstable",
    "chrome-dev",
    "chrome-for-testing",
    "chrome for testing",
    "chromium",
    "chromium-browser",
    "chromium-headless-shell",
    "chromium-dev",
    "chromium-for-testing",
    "chromium for testing",
    "google-chrome",
    "google-chrome-headless-shell",
    "google-chrome-stable",
    "google-chrome-beta",
    "google-chrome-canary",
    "google-chrome-unstable",
    "google-chrome-dev",
    "google-chrome-for-testing",
    "google chrome",
    "google chrome stable",
    "google chrome beta",
    "google chrome canary",
    "google chrome unstable",
    "google chrome dev",
    "google chrome for testing",
    "brave",
    "brave-browser",
    "brave-browser-stable",
    "brave browser stable",
    "brave-browser-beta",
    "brave-browser-dev",
    "brave-browser-nightly",
    "brave browser",
    "brave browser beta",
    "brave browser dev",
    "brave browser nightly",
    "vivaldi",
    "vivaldi-stable",
    "vivaldi-snapshot",
    "vivaldi snapshot",
    "opera",
    "opera-stable",
    "opera-beta",
    "opera-developer",
    "opera-unstable",
    "opera beta",
    "opera developer",
    "opera unstable",
    "msedge",
    "msedge-stable",
    "msedge-beta",
    "msedge-canary",
    "msedge-dev",
    "msedge-insider",
    "edge",
    "edge-stable",
    "edge-beta",
    "edge-canary",
    "edge-dev",
    "edge-insider",
    "microsoft-edge",
    "microsoft-edge-stable",
    "microsoft-edge-beta",
    "microsoft-edge-canary",
    "microsoft-edge-dev",
    "microsoft-edge-insider",
    "microsoft edge",
    "microsoft edge stable",
    "microsoft edge beta",
    "microsoft edge canary",
    "microsoft edge dev",
    "microsoft edge insider",
    "firefox",
    "firefox-esr",
    "firefox-beta",
    "firefox-nightly",
    "firefox-developer-edition",
    "firefox developer edition",
    "firefox beta",
    "librewolf",
    "waterfox",
    "mullvad-browser",
    "mullvad browser",
    "privacy-browser",
    "privacy browser",
    "zen-browser",
    "zen browser",
    "thorium-browser",
    "thorium browser",
];

pub(crate) fn browser_harness_is_browser_executable_name(executable: &str) -> bool {
    BROWSER_HARNESS_BROWSER_EXECUTABLE_NAMES.contains(&executable)
}

pub(crate) fn browser_harness_command_parts_for_browser_executable(
    executable: &str,
) -> Option<Vec<String>> {
    let executable = browser_harness_normalized_executable_name(executable);

    if browser_harness_is_browser_executable_name(&executable) {
        Some(vec![executable, "--headless".to_string()])
    } else {
        None
    }
}

/// Resolve the default browser-harness command from an availability probe.
///
/// A JavaScript runtime (`node`/`bun`/`deno`) is preferred over a real browser:
/// its process stdout is exactly the program's console output, which the
/// browser-harness contract (and every browser test) asserts on. A real browser
/// reproduces that stdout contract only under a DevTools driver, so it is selected
/// solely when no JS runtime is available. Pure over the injected availability
/// probe so the selection order can be unit-tested deterministically.
pub(crate) fn browser_harness_default_command_parts_from(
    is_available: impl Fn(&str) -> bool,
) -> Vec<String> {
    for runtime in ["node", "bun", "deno"] {
        if is_available(runtime) {
            return vec![runtime.to_string()];
        }
    }
    for candidate in BROWSER_HARNESS_BROWSER_EXECUTABLE_NAMES {
        if is_available(candidate) {
            if let Some(parts) = browser_harness_command_parts_for_browser_executable(candidate) {
                return parts;
            }
        }
    }
    vec!["node".to_string()]
}

pub(crate) fn browser_harness_default_command_parts() -> Vec<String> {
    static BROWSER_HARNESS_COMMAND: OnceLock<Vec<String>> = OnceLock::new();
    BROWSER_HARNESS_COMMAND
        .get_or_init(|| {
            browser_harness_default_command_parts_from(|executable| {
                Command::new(executable).arg("--version").output().is_ok()
            })
        })
        .clone()
}

/// Return the command used by browser smoke or future browser-runtime harnesses.
///
/// The helper accepts the same argv-style shell subset as [`split_command_spec`]
/// and falls back to the deterministic default host command when no override is
/// supplied.
pub fn browser_harness_command_parts_checked(command: Option<&str>) -> Result<Vec<String>, String> {
    if let Some(command) = command {
        let raw_command = command;
        let command = command.trim();
        if command.is_empty() {
            return Err(format!(
                "malformed {BROWSER_HARNESS_COMMAND_ENV} override: {raw_command:?}"
            ));
        }
        match split_command_spec(command) {
            Some(parts) if !parts.is_empty() => {
                if parts.first().is_some_and(|part| part.starts_with('-')) {
                    return Err(format!(
                        "malformed {BROWSER_HARNESS_COMMAND_ENV} override: {raw_command:?}"
                    ));
                }
                return Ok(parts);
            }
            _ => {
                return Err(format!(
                    "malformed {BROWSER_HARNESS_COMMAND_ENV} override: {raw_command:?}"
                ));
            }
        }
    }

    Ok(browser_harness_default_command_parts())
}

/// Return the command used by browser smoke or future browser-runtime harnesses.
///
/// This convenience wrapper preserves the historical infallible shape for tests
/// and other call sites that expect a guaranteed command vector.
pub fn browser_harness_command_parts_for(command: Option<&str>) -> Vec<String> {
    browser_harness_command_parts_checked(command).unwrap_or_else(|error| panic!("{error}"))
}

pub(crate) fn browser_harness_uses_html_entrypoint(executable: &str) -> bool {
    browser_harness_command_parts_for_browser_executable(executable).is_some()
}

/// Return the effective browser harness command using the configured environment override.
pub fn browser_harness_command_parts() -> Vec<String> {
    browser_harness_command_parts_for(std::env::var(BROWSER_HARNESS_COMMAND_ENV).ok().as_deref())
}

#[cfg(test)]
#[path = "command_tests.rs"]
mod command_tests;
