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

    libtest_mimic::run(&args, trials).exit_code()
}

#[cfg(test)]
#[path = "discover_tests.rs"]
mod discover_tests;
