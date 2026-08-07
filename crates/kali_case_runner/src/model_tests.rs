use super::*;

#[test]
fn a_single_step_case_parses_with_inline_step_fields() {
    let text = r#"
[source]
"main.js" = "console.log(1);\n"

[[case]]
name = "run"
args = ["run", "main.js"]
exit = "success"
stdout = "1\n"
"#;
    let parsed = parse_case_file(text).expect("parse");
    assert_eq!(parsed.source["main.js"], "console.log(1);\n");
    assert_eq!(parsed.case.len(), 1);
    assert_eq!(parsed.case[0].name, "run");
    assert!(parsed.case[0].step.is_empty());
    let inline = parsed.case[0].inline.as_ref().expect("inline step");
    assert_eq!(inline.args, vec!["run", "main.js"]);
    assert_eq!(inline.exit, Some(Exit::SUCCESS));
    assert_eq!(inline.stdout.as_deref(), Some("1\n"));
}

#[test]
fn a_multi_step_case_parses_its_steps_in_order() {
    let text = r#"
[[case]]
name = "bundle_and_harness"

  [[case.step]]
  kind = "cli"
  args = ["build", "--bundle"]
  exit = "success"

  [[case.step]]
  kind = "browser_bundle_harness"
  entry = "app"
  body = "await mod.f();"
  stdout_contains = ["1\n"]
"#;
    let parsed = parse_case_file(text).expect("parse");
    assert_eq!(parsed.case[0].step.len(), 2);
    assert_eq!(parsed.case[0].step[0].kind, StepKind::Cli);
    assert_eq!(parsed.case[0].step[1].kind, StepKind::BrowserBundleHarness);
    assert_eq!(parsed.case[0].step[1].entry.as_deref(), Some("app"));
}

#[test]
fn an_exact_exit_code_parses_as_a_code() {
    let text = r#"
[[case]]
name = "c"
args = ["run", "main.js"]
exit = 2
"#;
    let parsed = parse_case_file(text).expect("parse");
    let inline = parsed.case[0].inline.as_ref().expect("inline step");
    assert_eq!(inline.exit, Some(Exit::Code(2)));
}

#[test]
fn json_null_parses_as_a_list_of_dotted_paths() {
    let text = r#"
[[case]]
name = "c"
args = ["--output", "json", "check", "main.js"]
json_null = ["stdout", "stderr"]
"#;
    let parsed = parse_case_file(text).expect("parse");
    let inline = parsed.case[0].inline.as_ref().expect("inline step");
    assert_eq!(inline.json_null, vec!["stdout", "stderr"]);
}

// `json_null` shares `json`'s field-applicability rule (both read the
// step's captured process stdout, so both are `cli`/`browser_bundle_
// harness`-only) -- this pins that a `file_json` step, which never runs a
// process, rejects it the same way it already rejects `json`.
#[test]
fn a_file_json_step_rejects_json_null() {
    let text = r#"
[[case]]
name = "c"
kind = "file_json"
path = "o.json"
json_null = ["stderr"]
"#;
    let err = parse_case_file(text).expect_err("must reject json_null on a file_json step");
    assert!(
        err.contains("json_null"),
        "error must name the field: {err}"
    );
}

#[test]
fn dotted_json_keys_parse_into_a_nested_table() {
    let text = r#"
[[case]]
name = "c"
args = ["check", "main.ts"]
json.schemaVersion = 1
json.payload.artifactKind = "bundle"
"#;
    let parsed = parse_case_file(text).expect("parse");
    let json = parsed.case[0]
        .inline
        .as_ref()
        .unwrap()
        .json
        .as_ref()
        .expect("json");
    assert_eq!(json["schemaVersion"].as_integer(), Some(1));
    assert_eq!(json["payload"]["artifactKind"].as_str(), Some("bundle"));
}

// The format must not become a degradation vector: a typo'd key that silently
// asserts nothing is worse than no test at all (spec 5.10).
#[test]
fn an_unknown_key_is_a_hard_error_naming_the_key() {
    let text = r#"
[[case]]
name = "c"
args = ["run", "main.js"]
stdout_contain = ["oops"]
"#;
    let err = parse_case_file(text).expect_err("must reject unknown key");
    assert!(
        err.contains("stdout_contain"),
        "error must name the key: {err}"
    );
}

#[test]
fn a_case_file_with_no_cases_is_a_hard_error() {
    let text = r#"
[source]
"main.js" = "console.log(1);\n"
"#;
    let err = parse_case_file(text).expect_err("must reject zero cases");
    assert!(err.contains("no [[case]]"), "error must explain: {err}");
}

#[test]
fn a_matrix_axis_with_no_values_is_a_hard_error() {
    let text = r#"
[matrix]
ext = []

[[case]]
name = "c"
args = ["run", "main.js"]
"#;
    let err = parse_case_file(text).expect_err("must reject empty axis");
    assert!(err.contains("ext"), "error must name the axis: {err}");
}

// A case must declare exactly one of `[[case.step]]` or inline step fields
// (see model.rs's module doc comment for why `Case` is built from a raw
// `RawCase` with the residual keys manually converted, rather than a
// straight `#[serde(flatten)]` field) -- `parse_case_file` enforces "never
// neither" here and "never both" in the next test.
#[test]
fn a_case_with_no_step_and_no_inline_fields_is_a_hard_error() {
    let text = r#"
[[case]]
name = "empty"
"#;
    let err = parse_case_file(text).expect_err("must reject a case with no step");
    assert!(err.contains("empty"), "error must name the case: {err}");
    assert!(err.contains("no step"), "error must explain: {err}");
}

#[test]
fn a_case_mixing_step_list_and_inline_fields_is_a_hard_error() {
    let text = r#"
[[case]]
name = "mixed"
args = ["run", "main.js"]

  [[case.step]]
  kind = "cli"
  args = ["build"]
"#;
    let err = parse_case_file(text).expect_err("must reject mixing step forms");
    assert!(err.contains("mixed"), "error must name the case: {err}");
    assert!(err.contains("mixes"), "error must explain: {err}");
}

// The unknown-key guarantee must hold on *both* case-file syntaxes, not just
// the one covered above. `[[case.step]]` entries are ordinary (non-flatten)
// `Vec<Step>` elements, so they were never actually exposed to the
// flatten/deny_unknown_fields bug -- but a fix aimed at the inline path must
// not regress this path either, so it gets its own direct test.
#[test]
fn an_unknown_key_in_a_step_list_entry_is_a_hard_error_naming_the_key() {
    let text = r#"
[[case]]
name = "c"

  [[case.step]]
  kind = "cli"
  args = ["run", "main.js"]
  stdout_contain = ["oops"]
"#;
    let err = parse_case_file(text).expect_err("must reject unknown key in a step-list entry");
    assert!(
        err.contains("stdout_contain"),
        "error must name the key: {err}"
    );
}

// A known key with the wrong TOML value type must fail cleanly through the
// hand-written `toml::Value::Table(rest).try_into::<RawStep>()` conversion,
// not silently coerce, drop the field, or panic -- and the error must name
// the offending field, the same guarantee as the unknown-key tests above.
// `toml`'s error `Display` puts the field path on its own line ("in
// `args`"), which is what these assertions pin.
#[test]
fn an_inline_field_with_the_wrong_value_type_is_a_hard_error() {
    let text = r#"
[[case]]
name = "c"
args = "run"
"#;
    let err = parse_case_file(text).expect_err("must reject args as a string, not a list");
    assert!(err.contains("args"), "error must name the field: {err}");
}

#[test]
fn an_exit_value_of_the_wrong_type_is_a_hard_error() {
    let text = r#"
[[case]]
name = "c"
args = ["run", "main.js"]
exit = true
"#;
    let err = parse_case_file(text).expect_err("must reject a boolean exit value");
    assert!(err.contains("exit"), "error must name the field: {err}");
}

// The same wrong-type check on the `[[case.step]]` path, which goes through
// ordinary (non-flatten) struct deserialization rather than the manual
// conversion -- confirming both paths reject bad input the same way, and
// both name the field.
#[test]
fn a_step_list_entry_with_the_wrong_value_type_is_a_hard_error() {
    let text = r#"
[[case]]
name = "c"

  [[case.step]]
  args = "run"
"#;
    let err = parse_case_file(text).expect_err("must reject args as a string, not a list");
    assert!(err.contains("args"), "error must name the field: {err}");
}

// Task 9 derives libtest trial ids from case names; two cases sharing a name
// would make a failure report ambiguous about which case actually failed.
#[test]
fn duplicate_case_names_are_a_hard_error() {
    let text = r#"
[[case]]
name = "dup"
args = ["run", "a.js"]

[[case]]
name = "dup"
args = ["run", "b.js"]
"#;
    let err = parse_case_file(text).expect_err("must reject duplicate case names");
    assert!(err.contains("dup"), "error must name the duplicate: {err}");
}

// `Step` is flat and every field is independently optional, so nothing
// stops an author from writing an assertion that the step's `kind` can
// never evaluate -- e.g. `stdout_contains` on a `file_json` step, which
// never runs a process and so never produces stdout. That parses clean
// today unless `parse_case_file` cross-checks fields against `kind`; this
// is the exact case the reviewer found parsing successfully.
#[test]
fn a_file_json_step_rejects_process_output_assertions_naming_the_field() {
    let text = r#"
[[case]]
name = "c"
kind = "file_json"
path = "o.json"
stdout_contains = ["never checked?"]
"#;
    let err = parse_case_file(text)
        .expect_err("must reject stdout_contains on a file_json step, which has no stdout");
    assert!(
        err.contains("stdout_contains"),
        "error must name the field: {err}"
    );
    assert!(err.contains("c"), "error must name the case: {err}");
}

// Same guarantee, but with kind given explicitly and a cli-inapplicable
// field (`path`) present -- confirms the applicability check fires even
// when `kind` isn't defaulted.
#[test]
fn a_cli_step_rejects_file_json_only_fields() {
    let text = r#"
[[case]]
name = "c"
kind = "cli"
args = ["run", "main.js"]
path = "o.json"
"#;
    let err = parse_case_file(text).expect_err("must reject path on a cli step");
    assert!(err.contains("path"), "error must name the field: {err}");
}

#[test]
fn a_browser_bundle_harness_step_rejects_cli_only_fields() {
    let text = r#"
[[case]]
name = "c"
kind = "browser_bundle_harness"
entry = "app"
body = "await mod.f();"
args = ["run", "main.js"]
"#;
    let err = parse_case_file(text).expect_err("must reject args on a browser_bundle_harness step");
    assert!(err.contains("args"), "error must name the field: {err}");
}

// `kind` defaults to `cli` -- but only when no kind-specific field is
// present. An author who writes `entry`/`body` (browser_bundle_harness-only
// fields) and forgets `kind = "browser_bundle_harness"` must get an error,
// not a silently-misinterpreted `cli` step that ignores `entry`/`body`
// entirely. This is the reviewer's second reported case.
#[test]
fn a_step_with_browser_only_fields_and_no_explicit_kind_is_a_hard_error() {
    let text = r#"
[[case]]
name = "c"
entry = "app"
body = "await mod.f();"
stdout_contains = ["1"]
"#;
    let err = parse_case_file(text).expect_err("must reject entry/body without an explicit kind");
    assert!(err.contains("kind"), "error must explain: {err}");
}

// Symmetric case: `path`/`fields` (file_json-only fields) without an
// explicit `kind`.
#[test]
fn a_step_with_file_json_only_fields_and_no_explicit_kind_is_a_hard_error() {
    let text = r#"
[[case]]
name = "c"
path = "o.json"
fields.ok = true
"#;
    let err = parse_case_file(text).expect_err("must reject path/fields without an explicit kind");
    assert!(err.contains("kind"), "error must explain: {err}");
}

// The manual conversion (`toml::Value::Table(rest).try_into::<RawStep>()`)
// is hand-written, unlike everything else in this module, so it carries its
// own risk: a converter that silently drops a field would make every case
// file that relies on that field assert nothing, which is the exact class
// of bug this whole format exists to prevent. These three tests pin every
// one of `Step`'s fifteen fields through the inline (flatten + manual
// convert) path, split one case per `kind` since `finalize_step` now
// rejects kind-inapplicable fields -- a single case can no longer carry
// every field the way the original all-in-one version did.
#[test]
fn every_cli_step_field_round_trips_through_the_inline_conversion() {
    let text = r#"
[[case]]
name = "cli_kitchen_sink"
kind = "cli"
args = ["run", "main.js"]
env = { KALI_BROWSER_BUNDLE_HARNESS_COMMAND = "node" }
exit = "success"
stdout = "out\n"
stdout_contains = ["a"]
stdout_absent = ["b"]
stderr_contains = ["c"]
stderr_absent = ["d"]
json.schemaVersion = 1
json_null = ["stderr"]
"#;
    let parsed = parse_case_file(text).expect("parse");
    let step = parsed.case[0].inline.as_ref().expect("inline step");
    assert_eq!(step.kind, StepKind::Cli);
    assert_eq!(step.args, vec!["run", "main.js"]);
    assert_eq!(step.env["KALI_BROWSER_BUNDLE_HARNESS_COMMAND"], "node");
    assert_eq!(step.exit, Some(Exit::SUCCESS));
    assert_eq!(step.stdout.as_deref(), Some("out\n"));
    assert_eq!(step.stdout_contains, vec!["a"]);
    assert_eq!(step.stdout_absent, vec!["b"]);
    assert_eq!(step.stderr_contains, vec!["c"]);
    assert_eq!(step.stderr_absent, vec!["d"]);
    assert_eq!(
        step.json.as_ref().unwrap()["schemaVersion"].as_integer(),
        Some(1)
    );
    assert_eq!(step.json_null, vec!["stderr"]);
}

#[test]
fn every_file_json_step_field_round_trips_through_the_inline_conversion() {
    let text = r#"
[[case]]
name = "file_json_kitchen_sink"
kind = "file_json"
path = "out.json"
fields.ok = true
"#;
    let parsed = parse_case_file(text).expect("parse");
    let step = parsed.case[0].inline.as_ref().expect("inline step");
    assert_eq!(step.kind, StepKind::FileJson);
    assert_eq!(step.path.as_deref(), Some("out.json"));
    assert_eq!(step.fields.as_ref().unwrap()["ok"].as_bool(), Some(true));
}

#[test]
fn every_browser_bundle_harness_step_field_round_trips_through_the_inline_conversion() {
    let text = r#"
[[case]]
name = "browser_kitchen_sink"
kind = "browser_bundle_harness"
env = { KALI_BROWSER_BUNDLE_HARNESS_COMMAND = "node" }
exit = "failure"
stdout = "out\n"
stdout_contains = ["a"]
stdout_absent = ["b"]
stderr_contains = ["c"]
stderr_absent = ["d"]
json.schemaVersion = 2
json_null = ["stdout"]
entry = "app"
body = "await mod.f();"
"#;
    let parsed = parse_case_file(text).expect("parse");
    let step = parsed.case[0].inline.as_ref().expect("inline step");
    assert_eq!(step.kind, StepKind::BrowserBundleHarness);
    assert_eq!(step.env["KALI_BROWSER_BUNDLE_HARNESS_COMMAND"], "node");
    assert_eq!(step.exit, Some(Exit::FAILURE));
    assert_eq!(step.stdout.as_deref(), Some("out\n"));
    assert_eq!(step.stdout_contains, vec!["a"]);
    assert_eq!(step.stdout_absent, vec!["b"]);
    assert_eq!(step.stderr_contains, vec!["c"]);
    assert_eq!(step.stderr_absent, vec!["d"]);
    assert_eq!(
        step.json.as_ref().unwrap()["schemaVersion"].as_integer(),
        Some(2)
    );
    assert_eq!(step.json_null, vec!["stdout"]);
    assert_eq!(step.entry.as_deref(), Some("app"));
    assert_eq!(step.body.as_deref(), Some("await mod.f();"));
}
