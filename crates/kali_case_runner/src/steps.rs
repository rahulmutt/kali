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

/// How many times a spawn is retried after `ETXTBSY`, and the first backoff.
///
/// Ten doublings from 1ms is ~1s of total sleep in the worst case, which never
/// happens: the window this closes is the lifetime of a `fork`ed child's
/// duplicated file descriptors, measured in microseconds. The budget is large
/// because the cost of exhausting it is a flaky gate and the cost of a spare
/// retry is nothing.
const ETXTBSY_RETRIES: u32 = 10;
const ETXTBSY_FIRST_BACKOFF: std::time::Duration = std::time::Duration::from_millis(1);

/// Run `attempt`, retrying while it fails with `ETXTBSY` ("Text file busy").
///
/// WHY THIS EXISTS. The test suite writes an executable stub, `chmod +x`es it
/// and immediately execs it. On Linux that races: `Command::output` forks, and
/// between the fork and the exec the child holds duplicates of every descriptor
/// its parent had open -- including a WRITE descriptor on a stub some sibling
/// test thread is still creating. Exec'ing a file that any process holds open
/// for writing fails with `ETXTBSY`. Nothing is wrong with either side; the
/// kernel is reporting a transient overlap.
///
/// It is not hypothetical and it is not confined to one test. Reproduced at
/// `--test-threads=32` under eight spinning CPU hogs: 1 failure in 80 runs of
/// `steps::`, landing on
/// `an_ordinary_nested_relative_source_key_still_writes_normally` -- a different
/// test from the one that first exposed it, which is the point. Any `stub_bin`
/// test can draw it, so the fix belongs at the spawn, not in a test.
///
/// Retrying is safe here in the way retries usually are not: `ETXTBSY` is
/// raised by `execve` BEFORE the program runs, so no side effect of the command
/// can have happened. A retry cannot double anything.
fn retry_on_etxtbsy<T>(mut attempt: impl FnMut() -> std::io::Result<T>) -> std::io::Result<T> {
    let mut backoff = ETXTBSY_FIRST_BACKOFF;
    for _ in 0..ETXTBSY_RETRIES {
        match attempt() {
            Err(error) if error.kind() == std::io::ErrorKind::ExecutableFileBusy => {
                std::thread::sleep(backoff);
                backoff *= 2;
            }
            settled => return settled,
        }
    }
    // The last attempt is returned as-is, so an ETXTBSY that genuinely will not
    // clear still surfaces as itself rather than as a made-up error.
    attempt()
}

fn capture(mut command: Command, step: &Step) -> Result<Captured, String> {
    for (key, value) in &step.env {
        command.env(key, value);
    }
    let output = retry_on_etxtbsy(|| command.output())
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
    // `step.path` is substitution-eligible (`expand.rs`), so this must be
    // checked against the *substituted* value, exactly as `run_trial` checks
    // `[source]` keys. Without it an absolute path -- or one with a `..`
    // component -- reads a file outside the trial's temp dir. The read is
    // harmless on its own; the failure that matters is a trial that escapes
    // its sandbox, finds a real file, and *passes*.
    validate_source_key(rel)?;
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

/// Reject a trial-relative path that would escape the trial's temp dir: an
/// absolute path (`Path::join` discards the base entirely when the joined
/// operand is absolute), or a relative path with any `..` component
/// (`Path::join` does not normalise those, so the OS resolves the escape at
/// access time).
///
/// Two call sites, both of which join author-supplied text onto the temp dir:
/// `run_trial` for every `[source]` key (a write), and `run_file_json` for a
/// step's `path` (a read). The read side matters less obviously but not less:
/// it cannot clobber anything, but a path that escapes, resolves to a real
/// file, and satisfies the step's `fields` is a trial that reports green
/// having asserted against something outside its own sandbox.
///
/// This must run against the *substituted* text -- the value already sitting
/// in `trial.source` / `step.path` -- not just the raw text in the case file.
/// A key like `"${dir}/main.js"` is harmless as written but can expand to
/// `../x.js` once a matrix axis or constant is substituted in; these two call
/// sites are the last points every such path funnels through before it is
/// joined onto the temp dir, regardless of how the `Trial` was constructed, so
/// checking here (rather than only at parse time) is what actually closes the
/// escape.
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

/// Printed-rationale budget, in lines and in characters.
///
/// Both bounds are needed, because the corpus's rationales are long in two
/// different shapes. Measured over the 3,804 rationales in the 287 shipped
/// case files: the median is 1,014 characters but the median *line count* is
/// 1 -- most rationales are a single unwrapped paragraph, so a line cap alone
/// would fire on only 55 of them (the longest is 30 lines) while a 6,097-
/// character one-liner still buried the diff under ~76 wrapped terminal rows.
/// A character cap alone would cut the 30-line ones mid-structure. Together
/// they bound what a failure prints before the thing the reader came for --
/// the actual diff -- at roughly one screen: 1,018 of 3,804 rationales
/// (26.8%) print truncated, the rest print whole.
const RATIONALE_PRINT_LINES: usize = 15;
const RATIONALE_PRINT_CHARS: usize = 1_500;

/// The case file a trial came from, derived from its id (`family/file::case`,
/// with an optional `[axis=value,...]` matrix suffix on the stem). Printed
/// with a truncated rationale so the full prose is one `$EDITOR` away.
fn case_file_of(config: &RunnerConfig, trial: &Trial) -> String {
    let stem = trial
        .id
        .split_once("::")
        .map_or(&*trial.id, |(head, _)| head);
    let stem = stem.split_once('[').map_or(stem, |(head, _)| head);
    config
        .cases_dir
        .join(format!("{stem}.toml"))
        .display()
        .to_string()
}

/// Render `rationale` as the `  | `-prefixed block a failure prints, capped
/// at `RATIONALE_PRINT_LINES` lines and `RATIONALE_PRINT_CHARS` characters.
///
/// Truncation is of the *printed* form only -- the stored prose is untouched,
/// and the tail is never silently dropped: the elision line says how much was
/// held back and names the file holding all of it. The character cut is taken
/// at the last whitespace inside the budget where there is one, so it lands
/// between words rather than inside one.
fn render_rationale(rationale: &str, case_file: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    let mut used = 0usize;
    let mut truncated = false;
    for line in rationale.lines() {
        if kept.len() == RATIONALE_PRINT_LINES {
            truncated = true;
            break;
        }
        // +1 for the newline this line will occupy in the printed block.
        if used + line.len() + 1 > RATIONALE_PRINT_CHARS {
            let budget = RATIONALE_PRINT_CHARS.saturating_sub(used);
            let head = &line[..line
                .char_indices()
                .map(|(index, _)| index)
                .take_while(|index| *index <= budget)
                .last()
                .unwrap_or(0)];
            let head = head
                .rsplit_once(char::is_whitespace)
                .map_or(head, |(a, _)| a);
            if !head.is_empty() {
                kept.push(head);
            }
            truncated = true;
            break;
        }
        used += line.len() + 1;
        kept.push(line);
    }

    let mut out = String::from("  rationale:\n");
    for line in &kept {
        out.push_str("  | ");
        out.push_str(line);
        out.push('\n');
    }
    if truncated {
        out.push_str(&format!(
            "  | [...] rationale truncated for display at {} lines / {} chars -- full text in \
             {case_file}\n",
            RATIONALE_PRINT_LINES, RATIONALE_PRINT_CHARS
        ));
    }
    out
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
                message.push_str(&render_rationale(rationale, &case_file_of(config, trial)));
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
