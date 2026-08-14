//! Case-tree discovery and the libtest-mimic entry point.

use crate::expand::expand;
use crate::model::{parse_case_file, CaseFile};
use crate::steps::{run_trial, RunnerConfig};
use libtest_mimic::{Arguments, Failed, Trial as MimicTrial};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

fn collect(dir: &Path, prefix: &str, out: &mut Vec<(String, PathBuf)>) -> Result<(), String> {
    let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir)
        .map_err(|error| format!("cannot read {}: {error}", dir.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot read {}: {error}", dir.display()))?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            let nested = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            collect(&path, &nested, out)?;
        } else if name.eq_ignore_ascii_case(".toml") {
            // `Path::extension()` reports `None` for a leading-dot name with
            // no other dot, so `.toml` matches neither the `.toml` arm below
            // nor any author's expectation: it is a case file that is never
            // discovered, never run, and never mentioned. Everything else
            // this module does is about refusing exactly that outcome, so
            // refuse it here too rather than letting `extension()`'s
            // dotfile rule decide by accident.
            return Err(format!(
                "{}: a file named exactly `.toml` has no stem to name its trials with -- \
                 `Path::extension()` treats it as a dotfile, so it would be skipped in silence. \
                 Give it a name.",
                path.display()
            ));
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
        {
            // `file_stem`, not `trim_end_matches(".toml")`: the latter strips
            // every trailing `.toml` it finds, so `pad.toml` and
            // `pad.toml.toml` would both stem to `pad` -- a silent trial-id
            // collision (the same failure class Task 8 closed for duplicate
            // `[[case]]` names within one file, just across files here).
            // `file_stem` only ever strips the last extension.
            let stem = path
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or(name);
            let full = if prefix.is_empty() {
                stem
            } else {
                format!("{prefix}/{stem}")
            };
            out.push((full, path));
        }
    }
    Ok(())
}

pub fn discover(cases_dir: &Path) -> Result<Vec<(String, CaseFile)>, String> {
    if !cases_dir.exists() {
        return Err(format!(
            "case directory {} does not exist",
            cases_dir.display()
        ));
    }
    if !cases_dir.is_dir() {
        return Err(format!(
            "case directory {} is not a directory (found a file)",
            cases_dir.display()
        ));
    }
    let mut paths = Vec::new();
    collect(cases_dir, "", &mut paths)?;
    if paths.is_empty() {
        return Err(format!(
            "no case files found under {} -- refusing to report a green run over zero tests",
            cases_dir.display()
        ));
    }
    paths.sort_by(|a, b| a.0.cmp(&b.0));

    // `file_stem` (Minor 1) and case-insensitive extension matching
    // (Minor 2) are each correct alone, but together they reopen the
    // collision Minor 1 closed by a different route: `pad.toml` and
    // `pad.TOML` both stem to `pad`. That is the same failure class Task 8
    // made a hard parse error for duplicate `[[case]]` names within one
    // file -- ambiguous `--exact` filtering, an ambiguous gate failure
    // report that can't say which file broke -- just across files instead
    // of within one. Enforce the uniqueness explicitly here rather than
    // leaving it to emerge (or not) from how `collect`'s two matching rules
    // happen to interact; sorted-adjacency makes any duplicate a `windows(2)`
    // check away.
    for window in paths.windows(2) {
        let (stem_a, path_a) = &window[0];
        let (stem_b, path_b) = &window[1];
        if stem_a == stem_b {
            return Err(format!(
                "duplicate case-file stem `{stem_a}`: {} and {}",
                path_a.display(),
                path_b.display()
            ));
        }
    }

    let mut files = Vec::with_capacity(paths.len());
    for (stem, path) in paths {
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let parsed =
            parse_case_file(&text).map_err(|error| format!("{}: {error}", path.display()))?;
        files.push((stem, parsed));
    }
    Ok(files)
}

/// Refuse a run whose filter selects nothing.
///
/// `discover` already refuses a zero-*file* run, because "0 tests, ok" from a
/// mispointed case directory is a green run over nothing. A filter that
/// matches zero trials is the same failure with a different cause and the same
/// exit code: `cargo test -p kali_cli --test cases -- nonexistent_family/`
/// printed `0 passed; 0 failed; 5587 filtered out` and exited 0. That is not
/// hypothetical -- `cases/README.md` teaches filtering, and a CI lane pinned
/// to a family whose name later changes is exactly how a lane comes to test
/// nothing while staying green.
///
/// Only a *selector* triggers this. An absent (or empty) filter with no
/// `--skip` is the run-everything path and is never refused; `--list` is not
/// a test run at all and is left alone. `--ignored` on its own is likewise not
/// a selector -- asking for the ignored set and finding it empty is an answer,
/// not a mistake.
fn refuse_empty_selection(args: &Arguments, trials: &[MimicTrial]) -> Result<(), String> {
    if args.list {
        return Ok(());
    }
    let filter = args.filter.as_deref().unwrap_or("");
    if filter.is_empty() && args.skip.is_empty() {
        return Ok(());
    }
    if trials.iter().any(|trial| !args.is_filtered_out(trial)) {
        return Ok(());
    }
    let mut selector = Vec::new();
    if !filter.is_empty() {
        selector.push(format!("filter `{filter}`"));
    }
    for skip in &args.skip {
        selector.push(format!("--skip `{skip}`"));
    }
    Err(format!(
        "{} matched 0 of {} trials -- refusing to report a green run over zero tests. Check the \
         spelling (trial ids are `<family>/<file>::<case>`; `--list` prints them all).",
        selector.join(" with "),
        trials.len()
    ))
}

pub fn main_with(config: RunnerConfig) -> ExitCode {
    let args = Arguments::from_args();

    let files = match discover(&config.cases_dir) {
        Ok(files) => files,
        Err(error) => {
            eprintln!("case discovery failed: {error}");
            return ExitCode::FAILURE;
        }
    };

    let config = Arc::new(config);
    let mut trials = Vec::new();
    for (stem, file) in &files {
        let expanded = match expand(stem, file) {
            Ok(expanded) => expanded,
            Err(error) => {
                eprintln!("case expansion failed: {error}");
                return ExitCode::FAILURE;
            }
        };
        for trial in expanded {
            let config = Arc::clone(&config);
            let ignore = trial.ignore;
            let id = trial.id.clone();
            trials.push(
                MimicTrial::test(id, move || run_trial(&config, &trial).map_err(Failed::from))
                    .with_ignored_flag(ignore),
            );
        }
    }

    if let Err(error) = refuse_empty_selection(&args, &trials) {
        eprintln!("case selection failed: {error}");
        return ExitCode::FAILURE;
    }

    libtest_mimic::run(&args, trials).exit_code()
}

#[cfg(test)]
#[path = "discover_tests.rs"]
mod discover_tests;
