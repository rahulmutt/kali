//! Trial execution: one temp dir per trial, steps run in order, first failure
//! wins.

use crate::assertions::{check, check_json, Captured};
use crate::expand::Trial;
use crate::model::{Step, StepKind};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct RunnerConfig {
    pub kali_bin: PathBuf,
    pub cases_dir: PathBuf,
}

fn capture(mut command: Command, step: &Step) -> Result<Captured, String> {
    for (key, value) in &step.env {
        command.env(key, value);
    }
    let output = command
        .output()
        .map_err(|error| format!("failed to spawn: {error}"))?;
    Ok(Captured {
        code: output.status.code(),
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn run_cli(config: &RunnerConfig, dir: &Path, step: &Step) -> Result<(), String> {
    let mut command = Command::new(&config.kali_bin);
    command.current_dir(dir).args(&step.args);
    let captured = capture(command, step)?;
    check(step, &captured)
}

fn run_file_json(dir: &Path, step: &Step) -> Result<(), String> {
    let rel = step
        .path
        .as_deref()
        .ok_or_else(|| "file_json step requires `path`".to_string())?;
    let fields = step
        .fields
        .as_ref()
        .ok_or_else(|| "file_json step requires `fields`".to_string())?;
    let text = std::fs::read_to_string(dir.join(rel))
        .map_err(|error| format!("cannot read {rel}: {error}"))?;
    let actual: serde_json::Value =
        serde_json::from_str(&text).map_err(|error| format!("{rel} is not valid json: {error}"))?;
    check_json(fields, &actual)
}

/// Uses `browser_harness_command_parts_checked` rather than the infallible
/// `browser_harness_command_parts_for`, which panics on a malformed
/// `KALI_BROWSER_BUNDLE_HARNESS_COMMAND` override. Case files are
/// hand-authored across ~300 migrations, so a malformed override in a
/// step's `env` is a realistic input; it must fail the step with a
/// diagnosable message, not panic the trial (and, by extension, the whole
/// libtest-mimic process running it).
fn run_browser_bundle_harness(dir: &Path, step: &Step) -> Result<(), String> {
    let entry = step
        .entry
        .as_deref()
        .ok_or_else(|| "browser_bundle_harness step requires `entry`".to_string())?;
    let body = step
        .body
        .as_deref()
        .ok_or_else(|| "browser_bundle_harness step requires `body`".to_string())?;

    let script = kali_runtime_contract::browser_bundle_harness_script(entry, false, body);
    let harness_path = dir.join("browser-bundle-smoke.mjs");
    std::fs::write(&harness_path, script)
        .map_err(|error| format!("cannot write harness: {error}"))?;

    let override_command = step
        .env
        .get(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV)
        .map(String::as_str);
    let mut parts = kali_runtime_contract::browser_harness_command_parts_checked(override_command)
        .map_err(|error| format!("cannot resolve browser harness command: {error}"))?;
    if parts.is_empty() {
        return Err(
            "browser harness command resolved to an empty argv (this should never \
             happen -- browser_harness_command_parts_checked guarantees a non-empty \
             `Vec` on success)"
                .to_string(),
        );
    }
    let executable = parts.remove(0);
    let mut command = Command::new(executable);
    command.current_dir(dir).args(&parts).arg(&harness_path);
    let captured = capture(command, step)?;
    check(step, &captured)
}

/// Reject a `[source]` key that would write outside the trial's temp dir:
/// an absolute path (`Path::join` discards the base entirely when the joined
/// operand is absolute), or a relative path with any `..` component
/// (`Path::join` does not normalise those, so the OS resolves the escape at
/// write time).
///
/// This must run against the *substituted* key -- the one already sitting in
/// `trial.source` -- not just the raw text in the case file. A key like
/// `"${dir}/main.js"` is harmless as written but can expand to `../x.js`
/// once a matrix axis or constant is substituted in; `run_trial` is the last
/// point every source key funnels through before it is joined onto the temp
/// dir, regardless of how the `Trial` was constructed, so checking here
/// (rather than only at parse time) is what actually closes the escape.
///
/// Rejects rather than normalises: silently rewriting a case author's path
/// out from under them is its own surprise, and a case that meant to write
/// inside the trial dir should fail loudly, not succeed somewhere else.
fn validate_source_key(name: &str) -> Result<(), String> {
    let path = Path::new(name);
    if path.is_absolute() {
        return Err(format!(
            "source key `{name}` is an absolute path -- source files must be written relative \
             to the trial directory"
        ));
    }
    if path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(format!(
            "source key `{name}` escapes the trial directory via a `..` component -- rewrite \
             it to a path relative to the trial root"
        ));
    }
    Ok(())
}

pub fn run_trial(config: &RunnerConfig, trial: &Trial) -> Result<(), String> {
    let dir = tempfile::tempdir().map_err(|error| format!("tempdir: {error}"))?;

    for (name, body) in &trial.source {
        validate_source_key(name)?;
        let path = dir.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        }
        std::fs::write(&path, body).map_err(|error| format!("cannot write {name}: {error}"))?;
    }

    for (index, step) in trial.steps.iter().enumerate() {
        let result = match step.kind {
            StepKind::Cli => run_cli(config, dir.path(), step),
            StepKind::FileJson => run_file_json(dir.path(), step),
            StepKind::BrowserBundleHarness => run_browser_bundle_harness(dir.path(), step),
        };
        if let Err(detail) = result {
            let mut message = format!("step {} ({:?}) failed\n", index + 1, step.kind);
            if let Some(rationale) = &trial.rationale {
                message.push_str("  rationale:\n");
                for line in rationale.lines() {
                    message.push_str(&format!("  | {line}\n"));
                }
            }
            if !step.args.is_empty() {
                message.push_str(&format!("  argv: {:?}\n", step.args));
            }
            if !step.env.is_empty() {
                message.push_str(&format!("  env: {:?}\n", step.env));
            }
            message.push_str(&detail);
            return Err(message);
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "steps_tests.rs"]
mod steps_tests;
