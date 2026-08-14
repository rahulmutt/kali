use super::*;
use crate::{expand, parse_case_file};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

/// Write an executable stub that stands in for the `kali` binary.
fn stub_bin(dir: &std::path::Path, script: &str) -> std::path::PathBuf {
    let path = dir.join("stub-kali");
    let mut file = std::fs::File::create(&path).expect("create stub");
    writeln!(file, "#!/usr/bin/env bash").expect("write");
    write!(file, "{script}").expect("write");
    drop(file);
    let mut perms = std::fs::metadata(&path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms).expect("chmod");
    path
}

fn config_for(bin: std::path::PathBuf) -> RunnerConfig {
    RunnerConfig {
        kali_bin: bin,
        cases_dir: std::path::PathBuf::from("."),
    }
}

#[test]
fn a_cli_step_writes_the_source_and_asserts_on_output() {
    let home = tempfile::tempdir().expect("tempdir");
    let bin = stub_bin(home.path(), "cat main.js\n");
    let file = parse_case_file(
        r#"
[source]
"main.js" = "hello\n"

[[case]]
name = "run"
args = ["run", "main.js"]
exit = "success"
stdout = "hello\n"
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    run_trial(&config_for(bin), &trials[0]).expect("trial should pass");
}

#[test]
fn a_failing_step_reports_the_step_index_and_the_rationale() {
    let home = tempfile::tempdir().expect("tempdir");
    let bin = stub_bin(home.path(), "echo wrong\n");
    let file = parse_case_file(
        r#"
[[case]]
name = "run"
rationale = "pins the folded literal"
args = ["run", "main.js"]
stdout = "right\n"
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    let err = run_trial(&config_for(bin), &trials[0]).expect_err("must fail");
    assert!(err.contains("step 1"), "must name the step: {err}");
    assert!(
        err.contains("pins the folded literal"),
        "must print rationale: {err}"
    );
}

#[test]
fn later_steps_see_artifacts_written_by_earlier_steps() {
    let home = tempfile::tempdir().expect("tempdir");
    let bin = stub_bin(
        home.path(),
        "if [ \"$1\" = build ]; then mkdir -p app; echo '{\"apiSurface\":\"browser\"}' > app/app.meta.json; else cat app/app.meta.json; fi\n",
    );
    let file = parse_case_file(
        r#"
[[case]]
name = "build_then_read"

  [[case.step]]
  kind = "cli"
  args = ["build"]
  exit = "success"

  [[case.step]]
  kind = "file_json"
  path = "app/app.meta.json"
  fields = { apiSurface = "browser" }
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    run_trial(&config_for(bin), &trials[0]).expect("trial should pass");
}

#[test]
fn a_file_json_step_fails_when_the_file_is_absent() {
    let home = tempfile::tempdir().expect("tempdir");
    let bin = stub_bin(home.path(), "true\n");
    let file = parse_case_file(
        r#"
[[case]]
name = "c"

  [[case.step]]
  kind = "file_json"
  path = "app/app.meta.json"
  fields = { apiSurface = "browser" }
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    let err = run_trial(&config_for(bin), &trials[0]).expect_err("must fail");
    assert!(err.contains("app/app.meta.json"), "{err}");
}

#[test]
fn env_declared_on_a_step_reaches_the_child_process() {
    let home = tempfile::tempdir().expect("tempdir");
    let bin = stub_bin(home.path(), "echo \"$KALI_TEST_MARKER\"\n");
    let file = parse_case_file(
        r#"
[[case]]
name = "c"
args = ["run"]
env = { KALI_TEST_MARKER = "seen" }
stdout = "seen\n"
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    run_trial(&config_for(bin), &trials[0]).expect("trial should pass");
}

#[test]
fn a_browser_bundle_harness_step_requires_entry_and_body() {
    let home = tempfile::tempdir().expect("tempdir");
    let bin = stub_bin(home.path(), "true\n");
    let file = parse_case_file(
        r#"
[[case]]
name = "c"

  [[case.step]]
  kind = "browser_bundle_harness"
  entry = "app"
  exit = "success"
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    let err = run_trial(&config_for(bin), &trials[0]).expect_err("must fail");
    assert!(err.contains("body"), "must name the missing key: {err}");
}

// `run_trial` wraps `check`'s failure detail with its own two-space-prefixed
// lines (`  rationale:`, `  | `, `  argv:`, `  env:`). scripts/test-gate.sh
// parses `^    [A-Za-z_]` (a bare four-space indent) as a failed-test name, so
// the *composed* message -- not just run_trial's own lines -- must never
// produce one. This exercises the worst case: a multi-line rationale whose
// own continuation line is four-space indented, and captured stdout/stderr
// whose own lines are four-space indented too, alongside non-empty argv and
// env -- every piece that gets concatenated, in the shape most likely to
// trip the regex if the pipe-prefix argument ever stopped holding.
#[test]
fn the_composed_failure_text_never_uses_the_four_space_name_indent() {
    let home = tempfile::tempdir().expect("tempdir");
    let bin = stub_bin(
        home.path(),
        r#"printf 'wrong
    indented stdout line
'
printf 'stderr line
    indented stderr line
' 1>&2
"#,
    );
    let file = parse_case_file(
        r#"
[[case]]
name = "run"
rationale = """
pins the folded literal
    with an indented continuation line
"""
args = ["run", "main.js"]
env = { KALI_TEST_MARKER = "seen" }
stdout = "right\n"
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    let err = run_trial(&config_for(bin), &trials[0]).expect_err("must fail");
    for line in err.lines() {
        assert!(
            !(line.starts_with("    ")
                && line
                    .chars()
                    .nth(4)
                    .is_some_and(|c| c.is_alphabetic() || c == '_')),
            "line would be misparsed by test-gate.sh: {line:?}\nfull message:\n{err}"
        );
    }
    // Sanity: the worst-case inputs actually made it into the composed text,
    // so the loop above proved something rather than vacuously passing.
    assert!(err.contains("indented continuation line"), "{err}");
    assert!(err.contains("indented stdout line"), "{err}");
    assert!(err.contains("indented stderr line"), "{err}");
}

// Targets a writable absolute path (not e.g. `/etc/...`) so this reproduces
// the reviewer's actual hazard -- `std::fs::write` succeeding through the
// escape -- rather than merely tripping over an unrelated permission error,
// which would pass for the wrong reason with the guard removed.
#[test]
fn an_absolute_source_key_fails_the_trial_without_writing_through() {
    let home = tempfile::tempdir().expect("tempdir");
    let victim_dir = tempfile::tempdir().expect("tempdir");
    let victim_path = victim_dir.path().join("victim.txt");
    std::fs::write(&victim_path, "original\n").expect("seed victim");
    let victim_display = victim_path.display().to_string();
    let bin = stub_bin(home.path(), "true\n");
    let file = parse_case_file(&format!(
        r#"
[source]
"{victim_display}" = "clobbered\n"

[[case]]
name = "run"
args = ["run"]
exit = "success"
"#
    ))
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    let err = run_trial(&config_for(bin), &trials[0]).expect_err("must fail");
    assert!(
        err.contains(&victim_display),
        "must name the offending key: {err}"
    );
    let contents = std::fs::read_to_string(&victim_path).expect("victim still readable");
    assert_eq!(
        contents, "original\n",
        "must not have written through the absolute path"
    );
}

#[test]
fn a_relative_source_key_with_a_dotdot_component_fails_the_trial() {
    let home = tempfile::tempdir().expect("tempdir");
    let bin = stub_bin(home.path(), "true\n");
    let file = parse_case_file(
        r#"
[source]
"../escape.js" = "clobbered\n"

[[case]]
name = "run"
args = ["run"]
exit = "success"
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    let err = run_trial(&config_for(bin), &trials[0]).expect_err("must fail");
    assert!(
        err.contains("../escape.js"),
        "must name the offending key: {err}"
    );
}

// The check above the parse-time key alone would miss this: `${dir}` is
// harmless in the case file and only becomes `../` once the `dir` constant
// is substituted in during `expand`. This is what proves validation runs
// against the substituted key, not just the literal text an author wrote.
#[test]
fn a_source_key_that_only_escapes_after_substitution_fails() {
    let home = tempfile::tempdir().expect("tempdir");
    let bin = stub_bin(home.path(), "true\n");
    let file = parse_case_file(
        r#"
[constants]
dir = ".."

[source]
"${dir}/escape.js" = "clobbered\n"

[[case]]
name = "run"
args = ["run"]
exit = "success"
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    let err = run_trial(&config_for(bin), &trials[0]).expect_err("must fail");
    assert!(
        err.contains("../escape.js"),
        "must name the offending key: {err}"
    );
}

#[test]
fn an_ordinary_nested_relative_source_key_still_writes_normally() {
    let home = tempfile::tempdir().expect("tempdir");
    let bin = stub_bin(home.path(), "cat src/main.js\n");
    let file = parse_case_file(
        r#"
[source]
"src/main.js" = "hello\n"

[[case]]
name = "run"
args = ["run", "src/main.js"]
exit = "success"
stdout = "hello\n"
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    run_trial(&config_for(bin), &trials[0])
        .expect("trial should pass -- nested subdirectories are legitimate");
}

// `file_json`'s `path` is joined onto the trial dir exactly as a `[source]`
// key is, and is substitution-eligible in the same way, so it has to clear the
// same check. These two are the known positives for that: each points a
// `file_json` step at a file that (a) sits outside the trial directory, (b)
// really exists, and (c) contains exactly what the step's `fields` demand.
// Delete `validate_source_key(rel)?` from `run_file_json` and both trials go
// GREEN -- passing on an assertion satisfied entirely outside their sandbox --
// which is the failure this guard exists to make impossible.

#[test]
fn a_file_json_step_with_an_absolute_path_is_refused() {
    let home = tempfile::tempdir().expect("tempdir");
    let outside = tempfile::tempdir().expect("tempdir");
    let victim = outside.path().join("app.meta.json");
    std::fs::write(&victim, r#"{"apiSurface":"browser"}"#).expect("seed victim");
    let victim_display = victim.display().to_string();
    let bin = stub_bin(home.path(), "true\n");
    let file = parse_case_file(&format!(
        r#"
[[case]]
name = "c"

  [[case.step]]
  kind = "file_json"
  path = "{victim_display}"
  fields = {{ apiSurface = "browser" }}
"#
    ))
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    let err = run_trial(&config_for(bin), &trials[0]).expect_err("must fail");
    assert!(
        err.contains(&victim_display) && err.contains("absolute path"),
        "must refuse the absolute path by name: {err}"
    );
}

#[test]
fn a_file_json_step_with_a_dotdot_path_is_refused() {
    let home = tempfile::tempdir().expect("tempdir");
    // Created in `env::temp_dir()`, which is also where `tempfile::tempdir()`
    // puts the trial directory -- so `../<name>` really does resolve to this
    // file from inside the trial. `NamedTempFile` unlinks it on drop.
    let victim = tempfile::Builder::new()
        .prefix("kali-case-runner-escape-probe")
        .suffix(".json")
        .tempfile()
        .expect("victim tempfile");
    std::fs::write(victim.path(), r#"{"apiSurface":"browser"}"#).expect("seed victim");
    let victim_name = victim
        .path()
        .file_name()
        .expect("file name")
        .to_string_lossy()
        .into_owned();
    let bin = stub_bin(home.path(), "true\n");
    let file = parse_case_file(&format!(
        r#"
[[case]]
name = "c"

  [[case.step]]
  kind = "file_json"
  path = "../{victim_name}"
  fields = {{ apiSurface = "browser" }}
"#
    ))
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    let err = run_trial(&config_for(bin), &trials[0]).expect_err("must fail");
    assert!(
        err.contains(&victim_name) && err.contains(".."),
        "must refuse the escaping path by name: {err}"
    );
}

// `retry_on_etxtbsy` (I3). The race it closes cannot be forced deterministically
// through a real spawn, so the retry loop is exercised directly: these assert
// that it retries the one error kind it is meant to, gives up on any other, and
// surfaces a persistent ETXTBSY as itself rather than inventing a result.
#[test]
fn an_etxtbsy_spawn_is_retried_until_it_succeeds() {
    let mut attempts = 0;
    let out = retry_on_etxtbsy(|| {
        attempts += 1;
        if attempts < 3 {
            Err(std::io::Error::from(std::io::ErrorKind::ExecutableFileBusy))
        } else {
            Ok("ran")
        }
    })
    .expect("must succeed once the descriptor closes");
    assert_eq!(out, "ran");
    assert_eq!(attempts, 3, "must have retried twice, not spun or given up");
}

#[test]
fn a_non_etxtbsy_spawn_error_is_not_retried() {
    let mut attempts = 0;
    let err = retry_on_etxtbsy(|| {
        attempts += 1;
        Err::<(), _>(std::io::Error::from(std::io::ErrorKind::NotFound))
    })
    .expect_err("a missing binary must surface immediately");
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    assert_eq!(
        attempts, 1,
        "a non-transient error must not be retried at all"
    );
}

#[test]
fn a_persistent_etxtbsy_still_surfaces_as_itself() {
    let mut attempts = 0;
    let err = retry_on_etxtbsy(|| {
        attempts += 1;
        Err::<(), _>(std::io::Error::from(std::io::ErrorKind::ExecutableFileBusy))
    })
    .expect_err("must not invent a success");
    assert_eq!(err.kind(), std::io::ErrorKind::ExecutableFileBusy);
    assert_eq!(
        attempts,
        ETXTBSY_RETRIES as usize + 1,
        "the budget must be spent exactly once, then the last error returned"
    );
}

// `render_rationale` (F18). The stored prose is never altered -- only what a
// failure prints -- and the elision must always be audible and always name
// the file holding the rest.

#[test]
fn a_short_rationale_prints_whole_and_unmarked() {
    let out = render_rationale("one line\ntwo lines", "cases/x/y.toml");
    assert_eq!(out, "  rationale:\n  | one line\n  | two lines\n");
}

#[test]
fn a_rationale_over_the_line_budget_is_capped_and_says_so() {
    let long: String = (0..40)
        .map(|n| format!("line {n}\n"))
        .collect::<Vec<_>>()
        .concat();
    let out = render_rationale(&long, "cases/x/y.toml");
    let body = out.lines().filter(|l| l.starts_with("  | ")).count();
    assert_eq!(
        body,
        RATIONALE_PRINT_LINES + 1,
        "15 kept plus the marker: {out}"
    );
    assert!(out.contains("truncated for display"), "{out}");
    assert!(out.contains("cases/x/y.toml"), "must name the file: {out}");
    assert!(out.contains("line 14"), "{out}");
    assert!(!out.contains("line 20"), "the tail must not print: {out}");
}

// The corpus's median rationale is ONE line of ~1,014 characters, so a line
// cap alone would not fire on the shape that actually buries the diff. This
// is the character bound doing the work a line bound cannot.
#[test]
fn a_single_very_long_line_is_capped_by_the_character_budget() {
    let long = "word ".repeat(1400);
    let out = render_rationale(&long, "cases/x/y.toml");
    assert!(
        out.len() < long.len() / 2,
        "must be much shorter: {}",
        out.len()
    );
    assert!(out.contains("truncated for display"), "{out}");
    // Cut at a word boundary, not mid-token.
    let printed = out
        .lines()
        .find(|l| l.starts_with("  | word"))
        .expect("body line");
    assert!(
        printed.ends_with("word"),
        "must cut between words: {printed:?}"
    );
}

// Truncation is of the printed form only. The trial still holds every byte.
#[test]
fn truncating_the_printed_rationale_does_not_touch_the_stored_prose() {
    let home = tempfile::tempdir().expect("tempdir");
    let bin = stub_bin(home.path(), "echo wrong\n");
    let long = "sentence ".repeat(400);
    let file = parse_case_file(&format!(
        r#"
[[case]]
name = "run"
rationale = "{long}"
args = ["run", "main.js"]
stdout = "right\n"
"#
    ))
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    assert_eq!(
        trials[0].rationale.as_deref().map(str::len),
        Some(long.len()),
        "the stored rationale must be intact"
    );
    let err = run_trial(&config_for(bin), &trials[0]).expect_err("must fail");
    assert!(err.contains("truncated for display"), "{err}");
    assert!(err.len() < long.len(), "the printed form must be shorter");
}

// The composed message still has to clear test-gate.sh's `^    [A-Za-z_]`
// failed-test-name regex, including the new elision line.
#[test]
fn the_truncation_marker_never_uses_the_four_space_name_indent() {
    let out = render_rationale(&"x".repeat(9000), "cases/x/y.toml");
    for line in out.lines() {
        assert!(
            !(line.starts_with("    ")
                && line
                    .chars()
                    .nth(4)
                    .is_some_and(|c| c.is_alphabetic() || c == '_')),
            "line would be misparsed by test-gate.sh: {line:?}"
        );
    }
}

#[test]
fn the_case_file_hint_strips_a_matrix_suffix_from_the_trial_id() {
    let home = tempfile::tempdir().expect("tempdir");
    let config = RunnerConfig {
        kali_bin: home.path().join("kali"),
        cases_dir: std::path::PathBuf::from("crates/kali_cli/tests/cases"),
    };
    let file = parse_case_file(
        r#"
[matrix]
ext = ["js", "ts"]

[[case]]
name = "c"
args = ["run", "main.${ext}"]
exit = "success"
"#,
    )
    .expect("parse");
    let trials = expand("browser/y", &file).expect("expand");
    assert_eq!(trials[0].id, "browser/y[ext=js]::c");
    assert_eq!(
        case_file_of(&config, &trials[0]),
        "crates/kali_cli/tests/cases/browser/y.toml"
    );
}
