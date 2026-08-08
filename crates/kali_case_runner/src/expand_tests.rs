use super::*;
use crate::model::CountBound;
use crate::parse_case_file;

#[test]
fn a_case_file_with_no_matrix_yields_one_trial_per_case() {
    let file = parse_case_file(
        r#"
[source]
"main.js" = "console.log(1);\n"

[[case]]
name = "run"
args = ["run", "main.js"]

[[case]]
name = "check"
args = ["check", "main.js"]
"#,
    )
    .expect("parse");
    let trials = expand("string/x", &file).expect("expand");
    let ids: Vec<&str> = trials.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, vec!["string/x::run", "string/x::check"]);
}

#[test]
fn a_matrix_axis_substitutes_into_source_names_bodies_and_argv() {
    let file = parse_case_file(
        r#"
[matrix]
ext = ["js", "ts"]

[source]
"app.${ext}" = "// ${ext}\nconsole.log(1);\n"

[[case]]
name = "build"
args = ["build", "app.${ext}"]
"#,
    )
    .expect("parse");
    let trials = expand("browser/y", &file).expect("expand");
    assert_eq!(trials.len(), 2);
    assert_eq!(trials[0].id, "browser/y[ext=js]::build");
    assert!(trials[0].source.contains_key("app.js"));
    assert_eq!(trials[0].source["app.js"], "// js\nconsole.log(1);\n");
    assert_eq!(trials[0].steps[0].args, vec!["build", "app.js"]);
    assert_eq!(trials[1].id, "browser/y[ext=ts]::build");
    assert!(trials[1].source.contains_key("app.ts"));
}

#[test]
fn two_axes_form_a_cartesian_product_with_axes_sorted_by_name() {
    let file = parse_case_file(
        r#"
[matrix]
ext = ["js", "ts"]
api = ["browser", "node"]

[[case]]
name = "build"
args = ["build", "--api", "${api}", "app.${ext}"]
"#,
    )
    .expect("parse");
    let trials = expand("browser/z", &file).expect("expand");
    assert_eq!(trials.len(), 4);
    // `api` sorts before `ext`.
    assert_eq!(trials[0].id, "browser/z[api=browser,ext=js]::build");
    assert_eq!(trials[3].id, "browser/z[api=node,ext=ts]::build");
}

#[test]
fn constants_substitute_into_expected_strings() {
    let file = parse_case_file(
        r#"
[constants]
RULE_1 = "the discriminant is not a proven integer or string"

[[case]]
name = "float_discriminant"
args = ["run", "main.js"]
exit = "failure"
stderr_contains = ["E5506", "${RULE_1}"]
"#,
    )
    .expect("parse");
    let trials = expand("switch/fail_closed", &file).expect("expand");
    assert_eq!(
        trials[0].steps[0].stderr_contains,
        vec![
            "E5506",
            "the discriminant is not a proven integer or string"
        ]
    );
}

// An unresolved placeholder must never survive into a comparison, or the test
// silently asserts a literal `${...}` nobody will ever emit.
#[test]
fn an_unresolved_placeholder_is_a_hard_error_naming_it() {
    let file = parse_case_file(
        r#"
[[case]]
name = "c"
args = ["run", "main.js"]
stderr_contains = ["${NOPE}"]
"#,
    )
    .expect("parse");
    let err = expand("x/y", &file).expect_err("must reject unresolved");
    assert!(
        err.contains("NOPE"),
        "error must name the placeholder: {err}"
    );
}

#[test]
fn substitution_reaches_multi_step_cases_and_env_values() {
    let file = parse_case_file(
        r#"
[matrix]
ext = ["js"]

[constants]
CMD = "node"

[[case]]
name = "harness"

  [[case.step]]
  kind = "cli"
  args = ["run", "main.${ext}"]
  env = { KALI_BROWSER_BUNDLE_HARNESS_COMMAND = "${CMD}" }
"#,
    )
    .expect("parse");
    let trials = expand("browser/w", &file).expect("expand");
    assert_eq!(trials[0].steps[0].args, vec!["run", "main.js"]);
    assert_eq!(
        trials[0].steps[0].env["KALI_BROWSER_BUNDLE_HARNESS_COMMAND"],
        "node"
    );
}

#[test]
fn the_ignore_flag_and_rationale_carry_onto_every_expanded_trial() {
    let file = parse_case_file(
        r#"
[matrix]
ext = ["js", "ts"]

[[case]]
name = "c"
rationale = "why this exists"
ignore = true
args = ["run", "main.${ext}"]
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    assert!(trials.iter().all(|t| t.ignore));
    assert!(trials
        .iter()
        .all(|t| t.rationale.as_deref() == Some("why this exists")));
}

// The brief's own tests exercise `args`, `env` values, and `stderr_contains`.
// The remaining string-bearing `Step` fields -- `env` keys, `stdout`,
// `stdout_absent`, `stderr_absent`, and (on the other two `kind`s) `path`,
// `entry`, and `body` -- are just as capable of carrying a `${...}` that a
// forgetful `substitute_step` would silently leave untouched. Each of these
// is checked on its own, against the specific `kind` that actually accepts
// the field (see `finalize_step` in `model.rs`), so a regression in any one
// field is pinpointed rather than only detected in aggregate.

#[test]
fn substitution_reaches_stdout_and_stdout_absent() {
    let file = parse_case_file(
        r#"
[matrix]
ext = ["js"]

[[case]]
name = "c"
args = ["run", "main.${ext}"]
stdout = "built main.${ext}\n"
stdout_absent = ["warning: main.${ext}"]
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    assert_eq!(
        trials[0].steps[0].stdout.as_deref(),
        Some("built main.js\n")
    );
    assert_eq!(trials[0].steps[0].stdout_absent, vec!["warning: main.js"]);
}

#[test]
fn substitution_reaches_stderr() {
    let file = parse_case_file(
        r#"
[matrix]
ext = ["js"]

[[case]]
name = "c"
args = ["run", "main.${ext}"]
stderr = "warning: main.${ext}\n"
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    assert_eq!(
        trials[0].steps[0].stderr.as_deref(),
        Some("warning: main.js\n")
    );
}

#[test]
fn substitution_reaches_stderr_absent() {
    let file = parse_case_file(
        r#"
[matrix]
ext = ["js"]

[[case]]
name = "c"
args = ["run", "main.${ext}"]
stderr_absent = ["error: main.${ext}"]
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    assert_eq!(trials[0].steps[0].stderr_absent, vec!["error: main.js"]);
}

#[test]
fn substitution_reaches_env_keys_as_well_as_values() {
    let file = parse_case_file(
        r#"
[matrix]
ext = ["js"]

[[case]]
name = "c"

  [[case.step]]
  kind = "cli"
  args = ["run", "main.js"]
  env = { "KALI_${ext}_FLAG" = "on" }
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    assert_eq!(trials[0].steps[0].env["KALI_js_FLAG"], "on");
}

#[test]
fn substitution_reaches_file_json_path() {
    let file = parse_case_file(
        r#"
[matrix]
ext = ["js"]

[[case]]
name = "c"

  [[case.step]]
  kind = "file_json"
  path = "dist/app.${ext}.manifest.json"
  fields = { apiSurface = "browser" }
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    assert_eq!(
        trials[0].steps[0].path.as_deref(),
        Some("dist/app.js.manifest.json")
    );
}

#[test]
fn substitution_reaches_browser_bundle_harness_entry_and_body() {
    let file = parse_case_file(
        r#"
[matrix]
ext = ["js"]

[constants]
CALL = "mixedRootExpLog"

[[case]]
name = "c"

  [[case.step]]
  kind = "browser_bundle_harness"
  entry = "app_${ext}"
  body = "await mod.${CALL}();"
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    assert_eq!(trials[0].steps[0].entry.as_deref(), Some("app_js"));
    assert_eq!(
        trials[0].steps[0].body.as_deref(),
        Some("await mod.mixedRootExpLog();")
    );
}

// Minor from fix round 1: `stdout_contains` is the only `list`-routed field
// (same code path as `stdout_absent`/`stderr_contains`/`stderr_absent`) that
// had no field-specific test of its own, despite being one of the most-used
// assertion keys.
#[test]
fn substitution_reaches_stdout_contains() {
    let file = parse_case_file(
        r#"
[matrix]
ext = ["js"]

[[case]]
name = "c"
args = ["run", "main.${ext}"]
stdout_contains = ["compiled main.${ext}"]
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    assert_eq!(trials[0].steps[0].stdout_contains, vec!["compiled main.js"]);
}

// Fix round 1 (Important): the plan's `expand.rs` left `json`/`fields`
// unsubstituted (`step.json.clone()`), which spec §5.4 names as 2 of the 8
// assertion keys and §5.10's unresolved-placeholder invariant applies to
// unqualified. A matrix cell reusing one `.toml` case file must not leave a
// literal `${ext}` inside a `json`/`fields` expectation. Each test below
// targets one specific behaviour of the recursive substitution.

#[test]
fn a_matrix_axis_substitutes_into_a_json_expectation_per_cell() {
    let file = parse_case_file(
        r#"
[matrix]
ext = ["js", "ts"]

[[case]]
name = "build"
args = ["--output", "json", "build", "app.${ext}"]
json.payload.entry = "app.${ext}"
"#,
    )
    .expect("parse");
    let trials = expand("browser/y", &file).expect("expand");
    assert_eq!(trials.len(), 2);
    let json0 = trials[0].steps[0].json.as_ref().expect("json");
    assert_eq!(
        json0.get("payload").and_then(|p| p.get("entry")),
        Some(&toml::Value::String("app.js".to_string()))
    );
    let json1 = trials[1].steps[0].json.as_ref().expect("json");
    assert_eq!(
        json1.get("payload").and_then(|p| p.get("entry")),
        Some(&toml::Value::String("app.ts".to_string()))
    );
}

#[test]
fn a_matrix_axis_substitutes_into_a_file_json_fields_expectation() {
    let file = parse_case_file(
        r#"
[matrix]
ext = ["js"]

[[case]]
name = "c"

  [[case.step]]
  kind = "file_json"
  path = "dist/app.json"
  fields = { entry = "app.${ext}" }
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    let fields = trials[0].steps[0].fields.as_ref().expect("fields");
    assert_eq!(
        fields.get("entry"),
        Some(&toml::Value::String("app.js".to_string()))
    );
}

#[test]
fn an_unresolved_placeholder_inside_json_is_a_hard_error_naming_it() {
    let file = parse_case_file(
        r#"
[[case]]
name = "c"
args = ["run", "main.js"]
json.payload.entry = "${NOPE}"
"#,
    )
    .expect("parse");
    let err = expand("x/y", &file).expect_err("must reject unresolved");
    assert!(
        err.contains("NOPE"),
        "error must name the placeholder: {err}"
    );
}

#[test]
fn a_placeholder_in_a_json_key_substitutes_correctly() {
    let file = parse_case_file(
        r#"
[matrix]
ext = ["js"]

[[case]]
name = "c"
args = ["run", "main.js"]
json."entry_${ext}" = "ok"
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    let json = trials[0].steps[0].json.as_ref().expect("json");
    assert_eq!(
        json.get("entry_js"),
        Some(&toml::Value::String("ok".to_string()))
    );
    assert_eq!(json.get("entry_${ext}"), None);
}

#[test]
fn a_placeholder_nested_in_a_table_inside_an_array_substitutes() {
    let file = parse_case_file(
        r#"
[matrix]
ext = ["js"]

[[case]]
name = "c"
args = ["run", "main.js"]
json.payload.artifacts = [ { name = "app.${ext}" }, { name = "other" } ]
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    let json = trials[0].steps[0].json.as_ref().expect("json");
    let artifacts = json
        .get("payload")
        .and_then(|p| p.get("artifacts"))
        .and_then(|a| a.as_array())
        .expect("artifacts array");
    assert_eq!(
        artifacts[0].get("name"),
        Some(&toml::Value::String("app.js".to_string()))
    );
    assert_eq!(
        artifacts[1].get("name"),
        Some(&toml::Value::String("other".to_string()))
    );
}

// `json_null` is a plain `Vec<String>` of dotted paths, not a JSON tree, so
// it goes through the same `list()` helper as `stdout_contains` rather than
// `substitute_value` -- this is the field-specific test for that path,
// matching the pattern the other `list`-routed fields already have above.
#[test]
fn substitution_reaches_json_null() {
    let file = parse_case_file(
        r#"
[matrix]
ext = ["js"]

[[case]]
name = "c"
args = ["--output", "json", "check", "main.${ext}"]
json_null = ["payload.${ext}Stdout"]
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    assert_eq!(trials[0].steps[0].json_null, vec!["payload.jsStdout"]);
}

// A count claim is a table, not a string, so it is neither `list`-routed nor
// `substitute_value`-routed -- it gets its own pair of closures in
// `substitute_step`, and both of its string-bearing members need pinning. An
// unsubstituted `${...}` surviving into a *needle* is the dangerous case: the
// needle would then match nothing, and an `exact`-bounded claim could pass on
// a count of 0 while asserting nothing real.
#[test]
fn substitution_reaches_count_needles_and_json_count_paths() {
    let file = parse_case_file(
        r#"
[matrix]
ext = ["ts"]

[constants]
VALUE = "3"

[[case]]
name = "c"
args = ["run", "main.${ext}"]
stdout_count = [{ needle = "${VALUE}\n", at_least = 2 }]
json_count = [{ path = "payload.${ext}Stdout", needle = "${VALUE}\n", exact = 6 }]
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    let step = &trials[0].steps[0];
    assert_eq!(step.stdout_count[0].needle, "3\n");
    assert_eq!(step.stdout_count[0].bound, CountBound::AtLeast(2));
    assert_eq!(step.json_count[0].path, "payload.tsStdout");
    assert_eq!(step.json_count[0].needle, "3\n");
    assert_eq!(step.json_count[0].bound, CountBound::Exact(6));
}

#[test]
fn an_unresolved_placeholder_in_a_count_needle_is_a_hard_error_naming_it() {
    let file = parse_case_file(
        r#"
[[case]]
name = "c"
args = ["run", "main.js"]
stdout_count = [{ needle = "${nope}\n", at_least = 2 }]
"#,
    )
    .expect("parse");
    let err = expand("x/y", &file).expect_err("must reject the unresolved placeholder");
    assert!(
        err.contains("nope"),
        "error must name the placeholder: {err}"
    );
}

#[test]
fn non_string_json_leaves_survive_substitution_untouched() {
    let file = parse_case_file(
        r#"
[[case]]
name = "c"
args = ["run", "main.js"]
json.schemaVersion = 1
json.success = true
"#,
    )
    .expect("parse");
    let trials = expand("x/y", &file).expect("expand");
    let json = trials[0].steps[0].json.as_ref().expect("json");
    assert_eq!(json.get("schemaVersion"), Some(&toml::Value::Integer(1)));
    assert_eq!(json.get("success"), Some(&toml::Value::Boolean(true)));
}
