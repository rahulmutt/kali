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
