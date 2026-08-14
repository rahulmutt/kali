use super::*;

fn write(dir: &std::path::Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(path, text).expect("write");
}

const MINIMAL: &str = r#"
[[case]]
name = "c"
args = ["run", "main.js"]
exit = "success"
"#;

#[test]
fn discovery_returns_family_relative_stems_sorted() {
    let root = tempfile::tempdir().expect("tempdir");
    write(root.path(), "string/pad.toml", MINIMAL);
    write(root.path(), "array/at.toml", MINIMAL);
    write(root.path(), "string/repeat.toml", MINIMAL);
    let found = discover(root.path()).expect("discover");
    let stems: Vec<&str> = found.iter().map(|(stem, _)| stem.as_str()).collect();
    assert_eq!(stems, vec!["array/at", "string/pad", "string/repeat"]);
}

#[test]
fn non_toml_files_are_ignored() {
    let root = tempfile::tempdir().expect("tempdir");
    write(root.path(), "string/pad.toml", MINIMAL);
    write(root.path(), "string/README.md", "notes");
    let found = discover(root.path()).expect("discover");
    assert_eq!(found.len(), 1);
}

// A wrong discovery path must not report "0 tests, ok" and turn CI green.
#[test]
fn an_empty_case_tree_is_a_hard_error() {
    let root = tempfile::tempdir().expect("tempdir");
    let err = discover(root.path()).expect_err("must reject empty tree");
    assert!(err.contains("no case files"), "{err}");
}

#[test]
fn a_missing_case_directory_is_a_hard_error() {
    let root = tempfile::tempdir().expect("tempdir");
    let err = discover(&root.path().join("absent")).expect_err("must reject missing dir");
    assert!(err.contains("absent"), "{err}");
}

#[test]
fn a_malformed_case_file_errors_with_its_path() {
    let root = tempfile::tempdir().expect("tempdir");
    write(root.path(), "string/broken.toml", "[[case]]\nname = ");
    let err = discover(root.path()).expect_err("must reject malformed toml");
    assert!(err.contains("string/broken.toml"), "{err}");
}

// `trim_end_matches(".toml")` would strip every trailing `.toml`, collapsing
// `pad.toml` and `pad.toml.toml` to the same stem `pad` -- a silent trial-id
// collision. `file_stem` only ever strips the last extension, so the two
// stay distinct.
#[test]
fn a_repeated_toml_suffix_does_not_collapse_into_a_duplicate_stem() {
    let root = tempfile::tempdir().expect("tempdir");
    write(root.path(), "string/pad.toml", MINIMAL);
    write(root.path(), "string/pad.toml.toml", MINIMAL);
    let found = discover(root.path()).expect("discover");
    let mut stems: Vec<&str> = found.iter().map(|(stem, _)| stem.as_str()).collect();
    stems.sort();
    assert_eq!(stems, vec!["string/pad", "string/pad.toml"]);
}

// A miscased extension must not vanish silently -- that is the same failure
// class the empty-tree guard exists to prevent (a case file that never
// runs, unreported), just at per-file scale.
#[test]
fn a_miscased_toml_extension_is_still_discovered() {
    let root = tempfile::tempdir().expect("tempdir");
    write(root.path(), "string/UP.TOML", MINIMAL);
    let found = discover(root.path()).expect("discover");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, "string/UP");
}

// `Path::new(".toml").extension()` is `None` -- the dotfile rule -- so a file
// named exactly `.toml` matched neither the directory arm nor the `.toml`
// arm and vanished without a word. Same failure class as the empty-tree and
// miscased-extension guards: a case file that never runs, unreported.
#[test]
fn a_file_named_exactly_dot_toml_is_a_hard_error_rather_than_a_silent_skip() {
    let root = tempfile::tempdir().expect("tempdir");
    write(root.path(), "string/pad.toml", MINIMAL);
    write(root.path(), "string/.toml", MINIMAL);
    let err = discover(root.path()).expect_err("must refuse a file named `.toml`");
    assert!(err.contains(".toml"), "{err}");
    assert!(err.contains("skipped in silence"), "{err}");
}

// The dotfile rule is only a hazard for the exact name `.toml`. An ordinary
// hidden file is not a case file and must still be ignored quietly, or every
// editor swapfile in the tree becomes a hard error.
#[test]
fn an_ordinary_hidden_non_toml_file_is_still_ignored() {
    let root = tempfile::tempdir().expect("tempdir");
    write(root.path(), "string/pad.toml", MINIMAL);
    write(root.path(), "string/.gitkeep", "");
    let found = discover(root.path()).expect("discover");
    assert_eq!(found.len(), 1);
}

// Distinct from "does not exist": the path is present, just not a
// directory. Reporting "does not exist" for an existing file would send a
// case author looking for a typo that isn't there.
#[test]
fn a_case_directory_that_is_actually_a_file_is_a_hard_error_naming_it() {
    let root = tempfile::tempdir().expect("tempdir");
    let file_path = root.path().join("not_a_dir");
    std::fs::write(&file_path, "oops").expect("write");
    let err = discover(&file_path).expect_err("must reject a file where a directory is expected");
    assert!(!err.contains("does not exist"), "{err}");
    assert!(err.contains("not a directory"), "{err}");
}

// `refuse_empty_selection`. `discover`'s empty-tree guard closes "the case
// directory is wrong"; this closes the same green-over-nothing outcome
// reached by a filter that matches no trial. Reproduced before the fix:
// `cargo test -p kali_cli --test cases -- nonexistent_family/` printed
// `0 passed; 0 failed; 5587 filtered out` and exited 0.

fn sample_trials() -> Vec<MimicTrial> {
    vec![
        MimicTrial::test("string/pad::c", || Ok(())),
        MimicTrial::test("array/at::c", || Ok(())),
        MimicTrial::test("array/at::skipped", || Ok(())).with_ignored_flag(true),
    ]
}

fn args_from(argv: &[&str]) -> Arguments {
    let mut full = vec!["cases"];
    full.extend_from_slice(argv);
    Arguments::from_iter(full)
}

#[test]
fn a_filter_matching_zero_trials_is_refused() {
    let error = refuse_empty_selection(&args_from(&["nonexistent_family/"]), &sample_trials())
        .expect_err("a filter that selects nothing must not report a green run");
    assert!(error.contains("nonexistent_family/"), "{error}");
    assert!(error.contains("matched 0 of 3 trials"), "{error}");
}

#[test]
fn an_exact_filter_matching_zero_trials_is_refused() {
    let error = refuse_empty_selection(
        &args_from(&["--exact", "string/pad::typo"]),
        &sample_trials(),
    )
    .expect_err("an --exact filter that selects nothing must be refused");
    assert!(error.contains("string/pad::typo"), "{error}");
}

#[test]
fn a_skip_pattern_that_removes_every_trial_is_refused() {
    let error = refuse_empty_selection(&args_from(&["--skip", "::"]), &sample_trials())
        .expect_err("a --skip that leaves nothing must be refused");
    assert!(error.contains("--skip"), "{error}");
}

// The run-everything path is the common one and must stay untouched: no
// filter, no `--skip`, nothing to refuse.
#[test]
fn an_absent_filter_runs_everything() {
    refuse_empty_selection(&args_from(&[]), &sample_trials()).expect("bare run must be allowed");
}

// `cargo test -- ''` and some IDE runners pass an empty filter string, which
// selects every trial. That is the run-everything path spelled differently,
// not a mistake.
#[test]
fn an_empty_filter_string_runs_everything() {
    refuse_empty_selection(&args_from(&[""]), &sample_trials())
        .expect("an empty filter selects everything");
}

#[test]
fn a_filter_that_matches_is_allowed() {
    refuse_empty_selection(&args_from(&["array/"]), &sample_trials()).expect("must be allowed");
}

// A filter selecting only ignored trials has still selected something -- the
// run reports them as ignored, which is a real answer, not a silent zero.
#[test]
fn a_filter_matching_only_an_ignored_trial_is_allowed() {
    refuse_empty_selection(&args_from(&["array/at::skipped"]), &sample_trials())
        .expect("an ignored match is still a match");
}

// `--list` prints trial ids and runs nothing by construction; refusing it
// would break the very command the error message tells the reader to run.
#[test]
fn listing_with_a_filter_that_matches_nothing_is_not_refused() {
    refuse_empty_selection(
        &args_from(&["--list", "nonexistent_family/"]),
        &sample_trials(),
    )
    .expect("--list is not a test run");
}

// `file_stem` (Minor 1) and case-insensitive extension matching (Minor 2)
// are each correct alone, but together they reopen the exact collision
// Minor 1 closed, by a different route: `pad.toml` and `pad.TOML` both stem
// to `pad`. Must be a hard error naming both paths, not a silent duplicate
// trial id.
#[test]
fn a_case_insensitive_extension_collision_is_a_hard_error_naming_both_paths() {
    let root = tempfile::tempdir().expect("tempdir");
    write(root.path(), "string/pad.toml", MINIMAL);
    write(root.path(), "string/pad.TOML", MINIMAL);
    let err = discover(root.path()).expect_err("must reject the duplicate stem");
    assert!(err.contains("string/pad"), "{err}");
    assert!(err.contains("pad.toml"), "{err}");
    assert!(err.contains("pad.TOML"), "{err}");
}
