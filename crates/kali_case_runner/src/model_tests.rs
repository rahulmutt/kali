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

// `stderr` is exact-equality, symmetric with `stdout` (see `Step::stderr`'s
// doc comment for why it exists as a separate key from `stderr_contains`/
// `stderr_absent`).
#[test]
fn stderr_parses_as_an_exact_string() {
    let text = r#"
[[case]]
name = "c"
args = ["run", "main.js"]
stderr = ""
"#;
    let parsed = parse_case_file(text).expect("parse");
    let inline = parsed.case[0].inline.as_ref().expect("inline step");
    assert_eq!(inline.stderr.as_deref(), Some(""));
}

// `stderr` shares `stdout`'s field-applicability rule (both read the
// step's captured process output, so both are `cli`/`browser_bundle_
// harness`-only) -- this pins that a `file_json` step, which never runs a
// process, rejects it the same way it already rejects `stdout`.
#[test]
fn a_file_json_step_rejects_stderr() {
    let text = r#"
[[case]]
name = "c"
kind = "file_json"
path = "o.json"
stderr = ""
"#;
    let err = parse_case_file(text).expect_err("must reject stderr on a file_json step");
    assert!(err.contains("stderr"), "error must name the field: {err}");
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

// `stdout_count`/`json_count` are the third key added mid-migration (after
// `json_null` and `stderr`), for the `.matches(needle).count()` shape. Both
// spellings and both surfaces parse from one step.
#[test]
fn count_claims_parse_on_both_surfaces_and_in_both_spellings() {
    let text = r#"
[[case]]
name = "c"
args = ["run", "main.js"]
stdout_count = [
  { needle = "3\n", at_least = 2 },
  { needle = "ok 1", exact = 1 },
]
json_count = [{ path = "payload.stdout", needle = "1.2649110640673518", exact = 6 }]
"#;
    let parsed = parse_case_file(text).expect("parse");
    let inline = parsed.case[0].inline.as_ref().expect("inline step");
    assert_eq!(
        inline.stdout_count,
        vec![
            CountClaim {
                needle: "3\n".to_string(),
                bound: CountBound::AtLeast(2),
            },
            CountClaim {
                needle: "ok 1".to_string(),
                bound: CountBound::Exact(1),
            },
        ]
    );
    assert_eq!(
        inline.json_count,
        vec![JsonCountClaim {
            path: "payload.stdout".to_string(),
            needle: "1.2649110640673518".to_string(),
            bound: CountBound::Exact(6),
        }]
    );
}

// Every guard on a count claim -- `deny_unknown_fields`, the empty needle,
// `at_least = 0`, exactly-one-bound -- applies identically to both count
// keys, but each key reaches `count_needle`/`count_bound` through its *own*
// arm of `finalize_step` and carries its own `deny_unknown_fields` derive. A
// guard pinned against only one key is therefore held closed by nothing on
// the other: dropping `count_needle` from the `json_count` arm alone lets
// `json_count = [{ path = "stdout", needle = "", at_least = 2 }]` parse, and
// `str::matches("")` then returns `chars + 1`, so every such claim passes
// vacuously. Each guard below runs against both keys for that reason.
//
// The second element is whatever the key's claim table requires beyond the
// members under test -- `json_count` needs a `path`, `stdout_count` needs
// nothing.
const COUNT_KEYS: [(&str, &str); 2] = [("stdout_count", ""), ("json_count", "path = \"stdout\", ")];

/// Panics naming the key under test, so a guard that holds for one count key
/// but not the other reports *which* one regressed rather than just "must
/// reject".
fn expect_rejection(text: &str, key: &str, what: &str) -> String {
    match parse_case_file(text) {
        Ok(_) => panic!("`{key}` must reject {what}, but it parsed:\n{text}"),
        Err(error) => error,
    }
}

fn count_case(key: &str, extra: &str, claim: &str) -> String {
    format!(
        "[[case]]\nname = \"c\"\nargs = [\"run\", \"main.js\"]\n\
         {key} = [{{ {extra}{claim} }}]\n"
    )
}

// A claim table with no bound would parse into "count the needle, compare it
// against nothing" -- exactly the assert-nothing degradation this format
// exists to close. There is no defensible default to fall back on, so it is
// a hard error.
#[test]
fn a_count_claim_with_no_bound_is_a_hard_error() {
    for (key, extra) in COUNT_KEYS {
        let text = count_case(key, extra, r#"needle = "3\n""#);
        let err = expect_rejection(&text, key, "a claim with neither bound");
        assert!(err.contains(key), "{key}: must name the key: {err}");
        assert!(err.contains("at_least"), "{key}: must explain: {err}");
    }
}

#[test]
fn a_count_claim_setting_both_bounds_is_a_hard_error() {
    for (key, extra) in COUNT_KEYS {
        let text = count_case(key, extra, r#"needle = "3\n", at_least = 2, exact = 3"#);
        let err = expect_rejection(&text, key, "a claim with both bounds");
        assert!(err.contains(key), "{key}: must name the key: {err}");
        assert!(err.contains("exactly one"), "{key}: must explain: {err}");
    }
}

// `at_least = 0` holds against every possible output, so it is a claim that
// can never fail -- rejected for the same reason a typo'd key is. `exact = 0`
// is a real claim (a stricter `stdout_absent`) and stays legal.
#[test]
fn an_at_least_zero_count_claim_is_rejected_but_exact_zero_is_not() {
    for (key, extra) in COUNT_KEYS {
        let vacuous = count_case(key, extra, r#"needle = "3\n", at_least = 0"#);
        let err = expect_rejection(&vacuous, key, "a claim nothing can violate");
        assert!(err.contains("at_least = 0"), "{key}: must explain: {err}");

        let meaningful = count_case(key, extra, r#"needle = "3\n", exact = 0"#);
        let parsed = parse_case_file(&meaningful)
            .unwrap_or_else(|e| panic!("{key}: `exact = 0` is a falsifiable claim: {e}"));
        let inline = parsed.case[0].inline.as_ref().expect("inline step");
        let bound = match key {
            "stdout_count" => inline.stdout_count[0].bound,
            _ => inline.json_count[0].bound,
        };
        assert_eq!(bound, CountBound::Exact(0), "{key}");
    }
}

// Rust's `str::matches("")` matches at every character boundary, yielding
// `len + 1` -- a number no author writing `count() >= 2` ever meant.
// Rejecting the needle at parse time avoids both reproducing that surprise
// and special-casing it into a silent divergence from `str::matches`.
#[test]
fn an_empty_count_needle_is_a_hard_error() {
    for (key, extra) in COUNT_KEYS {
        let text = count_case(key, extra, r#"needle = "", at_least = 2"#);
        let err = expect_rejection(&text, key, "an empty needle");
        assert!(err.contains("needle"), "{key}: must name the field: {err}");
        assert!(err.contains(key), "{key}: must name the key: {err}");
    }
}

// The claim table itself carries `deny_unknown_fields`, so a misspelt bound
// cannot be read as "a claim with no bound at all" (which the check above
// would then have to catch by accident) or, worse, silently dropped.
#[test]
fn an_unknown_key_inside_a_count_claim_is_a_hard_error_naming_it() {
    for (key, extra) in COUNT_KEYS {
        let text = count_case(key, extra, r#"needle = "3\n", atleast = 2"#);
        let err = expect_rejection(&text, key, "a misspelt bound");
        assert!(err.contains("atleast"), "{key}: must name the key: {err}");
    }
}

// `stdout_count` and `json_count` share the applicability rule of the keys
// they extend (`stdout_contains` and `json`/`json_null`): all read the step's
// captured process output, so a `file_json` step -- which never runs a
// process -- must reject them rather than parse a claim it can never
// evaluate.
#[test]
fn a_file_json_step_rejects_both_count_keys() {
    let stdout_side = r#"
[[case]]
name = "c"
kind = "file_json"
path = "o.json"
stdout_count = [{ needle = "3\n", at_least = 2 }]
"#;
    let err =
        parse_case_file(stdout_side).expect_err("must reject stdout_count on a file_json step");
    assert!(err.contains("stdout_count"), "must name the field: {err}");

    let json_side = r#"
[[case]]
name = "c"
kind = "file_json"
path = "o.json"
json_count = [{ path = "stdout", needle = "3\n", at_least = 2 }]
"#;
    let err = parse_case_file(json_side).expect_err("must reject json_count on a file_json step");
    assert!(err.contains("json_count"), "must name the field: {err}");
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
exit = "success"

  [[case.step]]
  kind = "cli"
  args = ["build"]
  exit = "success"
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

// A step carrying two kinds' worth of fields has no kind to default to, and
// the message must say so rather than naming whichever one happens to be
// checked first. `path` is file_json-only; `verdict` is oracle-only.
#[test]
fn a_step_mixing_two_kinds_worth_of_fields_without_an_explicit_kind_is_a_hard_error() {
    let text = r#"
[[case]]
name = "c"
path = "o.json"
verdict = "silent"
"#;
    let err = parse_case_file(text).expect_err("must reject a two-kind step without a kind");
    assert!(
        err.contains("more than one kind"),
        "error must explain: {err}"
    );
}

// The manual conversion (`toml::Value::Table(rest).try_into::<RawStep>()`)
// is hand-written, unlike everything else in this module, so it carries its
// own risk: a converter that silently drops a field would make every case
// file that relies on that field assert nothing, which is the exact class
// of bug this whole format exists to prevent. These three tests, plus
// `an_oracle_step_parses_its_four_fields` below, pin every one of `Step`'s
// twenty-two fields through the inline (flatten + manual convert) path,
// split one case per `kind` since `finalize_step` now
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
stdout_count = [{ needle = "e", at_least = 2 }]
stderr = "err\n"
stderr_contains = ["c"]
stderr_absent = ["d"]
json.schemaVersion = 1
json_null = ["stderr"]
json_count = [{ path = "payload.stdout", needle = "f", exact = 3 }]
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
    assert_eq!(step.stdout_count, vec![at_least_claim("e", 2)]);
    assert_eq!(step.stderr.as_deref(), Some("err\n"));
    assert_eq!(step.stderr_contains, vec!["c"]);
    assert_eq!(step.stderr_absent, vec!["d"]);
    assert_eq!(
        step.json.as_ref().unwrap()["schemaVersion"].as_integer(),
        Some(1)
    );
    assert_eq!(step.json_null, vec!["stderr"]);
    assert_eq!(
        step.json_count,
        vec![JsonCountClaim {
            path: "payload.stdout".to_string(),
            needle: "f".to_string(),
            bound: CountBound::Exact(3),
        }]
    );
}

fn at_least_claim(needle: &str, n: usize) -> CountClaim {
    CountClaim {
        needle: needle.to_string(),
        bound: CountBound::AtLeast(n),
    }
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
stdout_count = [{ needle = "e", at_least = 2 }]
stderr = "err\n"
stderr_contains = ["c"]
stderr_absent = ["d"]
json.schemaVersion = 2
json_null = ["stdout"]
json_count = [{ path = "stdout", needle = "f", at_least = 4 }]
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
    assert_eq!(step.stderr.as_deref(), Some("err\n"));
    assert_eq!(step.stderr_contains, vec!["c"]);
    assert_eq!(step.stderr_absent, vec!["d"]);
    assert_eq!(
        step.json.as_ref().unwrap()["schemaVersion"].as_integer(),
        Some(2)
    );
    assert_eq!(step.json_null, vec!["stdout"]);
    assert_eq!(step.stdout_count, vec![at_least_claim("e", 2)]);
    assert_eq!(
        step.json_count,
        vec![JsonCountClaim {
            path: "stdout".to_string(),
            needle: "f".to_string(),
            bound: CountBound::AtLeast(4),
        }]
    );
    assert_eq!(step.entry.as_deref(), Some("app"));
    assert_eq!(step.body.as_deref(), Some("await mod.f();"));
}

// A step that runs a process and declares no assertion parses clean and
// passes unconditionally -- `kali` could segfault and the trial would still
// report green. The corpus holds zero such steps, so these tests pin a
// discipline that already holds rather than fixing a live break. Delete the
// `asserts` guard in `finalize_step` and the first two go green (the file
// parses), which is what makes them known positives.

#[test]
fn a_cli_step_with_no_assertion_key_is_a_hard_error() {
    let text = r#"
[[case]]
name = "asserts_nothing"
args = ["run", "main.js"]
"#;
    let err = parse_case_file(text).expect_err("must reject a step that asserts nothing");
    assert!(err.contains("asserts_nothing"), "must name the case: {err}");
    assert!(err.contains("declares no assertion"), "{err}");
    assert!(
        err.contains("pass unconditionally"),
        "must say why it matters: {err}"
    );
}

#[test]
fn a_browser_bundle_harness_step_with_no_assertion_key_is_a_hard_error() {
    let text = r#"
[[case]]
name = "asserts_nothing"

  [[case.step]]
  kind = "browser_bundle_harness"
  entry = "app"
  body = "await mod.f();"
"#;
    let err = parse_case_file(text).expect_err("must reject a step that asserts nothing");
    assert!(err.contains("declares no assertion"), "{err}");
    assert!(
        err.contains("browser_bundle_harness"),
        "must name the kind: {err}"
    );
}

// A guard that can only fail is worth as little as one that cannot: each of
// the eleven assertion keys must be enough on its own, or the guard is
// quietly demanding a *particular* key rather than any claim at all.
#[test]
fn any_single_assertion_key_satisfies_the_guard() {
    for key in [
        r#"exit = "success""#,
        r#"stdout = "x""#,
        r#"stdout_contains = ["x"]"#,
        r#"stdout_absent = ["x"]"#,
        r#"stdout_count = [{ needle = "x", at_least = 1 }]"#,
        r#"stderr = "x""#,
        r#"stderr_contains = ["x"]"#,
        r#"stderr_absent = ["x"]"#,
        r#"json = { schemaVersion = 2 }"#,
        r#"json_null = ["stderr"]"#,
        r#"json_count = [{ path = "stdout", needle = "x", at_least = 1 }]"#,
    ] {
        let text = format!("[[case]]\nname = \"c\"\nargs = [\"run\"]\n{key}\n");
        parse_case_file(&text).unwrap_or_else(|error| panic!("`{key}` must suffice: {error}"));
    }
}

// `file_json` is deliberately outside the guard: `fields` is its only
// assertion key and `run_file_json` already refuses a step without it, so the
// step can never reach a passing outcome having asserted nothing. Pinned so a
// later widening of the guard to all kinds has to confront this on purpose.
#[test]
fn a_file_json_step_is_not_subject_to_the_assertion_guard() {
    let text = r#"
[[case]]
name = "c"

  [[case.step]]
  kind = "file_json"
  path = "app/app.meta.json"
  fields = { apiSurface = "browser" }
"#;
    parse_case_file(text).expect("file_json's `fields` is its assertion");
}

// `check_bindings_are_referenced`. The corpus has zero violations of any of
// these, so every test below is a constructed known positive; delete the
// guard and each of the first four goes green.

#[test]
fn an_unreferenced_constant_is_a_hard_error() {
    let text = r#"
[constants]
UNUSED_NOTE = "E5506"

[[case]]
name = "c"
args = ["run"]
exit = "success"
"#;
    let err = parse_case_file(text).expect_err("must reject a dead constant");
    assert!(err.contains("UNUSED_NOTE"), "must name it: {err}");
    assert!(err.contains("never referenced"), "{err}");
}

#[test]
fn an_unreferenced_matrix_axis_is_a_hard_error() {
    let text = r#"
[matrix]
ext = ["js", "ts"]

[[case]]
name = "c"
args = ["run", "main.js"]
exit = "success"
"#;
    let err = parse_case_file(text).expect_err("must reject a dead axis");
    assert!(err.contains("ext"), "must name it: {err}");
    assert!(err.contains("byte-identical trials"), "{err}");
}

// `expand` inserts axis values over the constants, so the axis always wins.
// The constant looks referenced -- `${ext}` is right there -- but is not.
#[test]
fn a_constant_shadowed_by_a_matrix_axis_is_a_hard_error_that_says_so() {
    let text = r#"
[constants]
ext = "E5506"

[matrix]
ext = ["js", "ts"]

[[case]]
name = "c"
args = ["main.${ext}"]
exit = "success"
"#;
    let err = parse_case_file(text).expect_err("must reject an unreachable constant");
    assert!(
        err.contains("shadowed"),
        "must say why, not just 'unused': {err}"
    );
    assert!(!err.contains("never referenced"), "{err}");
}

// `substitute` is single-pass over `file.constants`, so `B = "${A}"` leaves a
// literal `${A}` in the expanded text -- `A` is genuinely dead, and counting
// B's mention of it would reopen the audit channel through one indirection.
#[test]
fn a_reference_from_another_constants_value_does_not_count() {
    let text = r#"
[constants]
A = "E5506"
B = "${A}"

[source]
"main.js" = "${B}"

[[case]]
name = "c"
args = ["run"]
exit = "success"
"#;
    let err = parse_case_file(text).expect_err("must reject the dead constant");
    assert!(err.contains("`A`"), "must name A: {err}");
}

// `expand` never substitutes `rationale` or a case `name`, so a `${X}` there
// is prose. If it counted, the dead-constant channel would be one sentence
// from working again.
#[test]
fn a_reference_from_a_rationale_does_not_count() {
    let text = r#"
[constants]
UNUSED_NOTE = "E5506"

[[case]]
name = "c"
rationale = "see ${UNUSED_NOTE}"
args = ["run"]
exit = "success"
"#;
    let err = parse_case_file(text).expect_err("must reject the dead constant");
    assert!(err.contains("UNUSED_NOTE"), "{err}");
}

// `matrix_cells` uses axis values raw -- they are never substituted.
#[test]
fn a_reference_from_a_matrix_axis_value_does_not_count() {
    let text = r#"
[constants]
UNUSED_NOTE = "E5506"

[matrix]
ext = ["${UNUSED_NOTE}"]

[[case]]
name = "c"
args = ["main.${ext}"]
exit = "success"
"#;
    let err = parse_case_file(text).expect_err("must reject the dead constant");
    assert!(err.contains("UNUSED_NOTE"), "{err}");
}

// The control in the other direction, and the reason the guard is
// reference-based rather than a blanket ban on `[constants]`: every field
// `expand::substitute_step` touches is a real reference site. Driven per
// field so a field dropped from `collect_step_placeholders` fails here rather
// than silently turning a live constant into a false "dead" one.
#[test]
fn a_reference_from_any_substituted_field_counts() {
    for (field, snippet) in [
        ("args", "args = [\"${P}\"]\nexit = \"success\""),
        (
            "env",
            "args = [\"run\"]\nexit = \"success\"\nenv = { K = \"${P}\" }",
        ),
        ("stdout", r#"stdout = "${P}""#),
        ("stdout_contains", r#"stdout_contains = ["${P}"]"#),
        ("stdout_absent", r#"stdout_absent = ["${P}"]"#),
        (
            "stdout_count",
            r#"stdout_count = [{ needle = "${P}", at_least = 1 }]"#,
        ),
        ("stderr", r#"stderr = "${P}""#),
        ("stderr_contains", r#"stderr_contains = ["${P}"]"#),
        ("stderr_absent", r#"stderr_absent = ["${P}"]"#),
        ("json", r#"json = { k = "${P}" }"#),
        ("json-key", r#"json = { "${P}" = "v" }"#),
        ("json_null", r#"json_null = ["${P}"]"#),
        (
            "json_count",
            r#"json_count = [{ path = "a", needle = "${P}", at_least = 1 }]"#,
        ),
    ] {
        let text = format!("[constants]\nP = \"v\"\n\n[[case]]\nname = \"c\"\n{snippet}\n");
        parse_case_file(&text)
            .unwrap_or_else(|error| panic!("a `${{P}}` in {field} is a real reference: {error}"));
    }
    // The `file_json`- and `browser_bundle_harness`-only fields, which cannot
    // share the `cli` shape above.
    for (field, snippet) in [
        (
            "path",
            "  kind = \"file_json\"\n  path = \"${P}\"\n  fields = { k = \"v\" }",
        ),
        (
            "fields",
            "  kind = \"file_json\"\n  path = \"a.json\"\n  fields = { k = \"${P}\" }",
        ),
        (
            "entry",
            "  kind = \"browser_bundle_harness\"\n  entry = \"${P}\"\n  body = \"x\"\n  exit = \"success\"",
        ),
        (
            "body",
            "  kind = \"browser_bundle_harness\"\n  entry = \"a\"\n  body = \"${P}\"\n  exit = \"success\"",
        ),
        (
            "register_entry",
            "  kind = \"oracle\"\n  register_entry = \"${P}\"\n  program = \"r.js\"\n  verdict = \"silent\"",
        ),
        (
            "program",
            "  kind = \"oracle\"\n  register_entry = \"R-1\"\n  program = \"${P}.js\"\n  verdict = \"silent\"",
        ),
    ] {
        let text =
            format!("[constants]\nP = \"v\"\n\n[[case]]\nname = \"c\"\n\n  [[case.step]]\n{snippet}\n");
        parse_case_file(&text)
            .unwrap_or_else(|error| panic!("a `${{P}}` in {field} is a real reference: {error}"));
    }
    // `[source]` keys and values.
    for (field, snippet) in [
        ("source-value", "[source]\n\"main.js\" = \"${P}\""),
        ("source-key", "[source]\n\"${P}.js\" = \"x\""),
    ] {
        let text = format!(
            "[constants]\nP = \"v\"\n\n{snippet}\n\n[[case]]\nname = \"c\"\nargs = [\"run\"]\nexit = \"success\"\n"
        );
        parse_case_file(&text)
            .unwrap_or_else(|error| panic!("a `${{P}}` in {field} is a real reference: {error}"));
    }
}

// The `${dollar}` escape (`dollar = "$"`, written `${dollar}{expr}` to emit a
// literal `${` into fixture text) is the corpus's one idiomatic constant. It
// must read as referenced, or every file using it stops parsing.
#[test]
fn the_dollar_escape_idiom_counts_as_a_reference() {
    let text = r#"
[constants]
dollar = "$"

[source]
"app.ts" = "console.log(`v: ${dollar}{7 / 2}`);\n"

[[case]]
name = "c"
args = ["run", "app.ts"]
exit = "success"
"#;
    parse_case_file(text).expect("`${dollar}` is a reference");
}

#[test]
fn an_oracle_step_parses_its_four_fields() {
    let text = r#"
[[case]]
name = "c"
kind = "oracle"
register_entry = "R-13"
program = "r13.js"
verdict = "silent"
timeout_ms = 5000
"#;
    let file = parse_case_file(text).expect("parses");
    let step = file.case[0].inline.as_ref().expect("inline step");
    assert_eq!(step.kind, StepKind::Oracle);
    assert_eq!(step.register_entry.as_deref(), Some("R-13"));
    assert_eq!(step.program.as_deref(), Some("r13.js"));
    assert_eq!(step.verdict, Some(kali_blast_radius::Verdict::Silent));
    assert_eq!(step.timeout_ms, Some(5000));
}

#[test]
fn oracle_fields_without_an_explicit_kind_are_rejected() {
    // Same rule browser_bundle_harness follows: a forgotten `kind` must not
    // silently become a `cli` step that ignores the fields entirely.
    let text = r#"
[[case]]
name = "c"
program = "r13.js"
verdict = "silent"
"#;
    let error = parse_case_file(text).expect_err("must demand an explicit kind");
    assert!(error.contains("oracle"), "error names the kind: {error}");
}

#[test]
fn an_oracle_step_declaring_stdout_assertions_is_rejected() {
    // An oracle step asserts a derived class. A `stdout` claim on it would
    // never be evaluated -- parses clean, asserts nothing.
    let text = r#"
[[case]]
name = "c"
kind = "oracle"
register_entry = "R-13"
program = "r13.js"
verdict = "silent"
stdout = "1\n"
"#;
    let error = parse_case_file(text).expect_err("must reject inapplicable assertions");
    assert!(error.contains("stdout"), "error names the field: {error}");
}

#[test]
fn an_oracle_step_missing_verdict_is_rejected() {
    let text = r#"
[[case]]
name = "c"
kind = "oracle"
register_entry = "R-13"
program = "r13.js"
"#;
    let error = parse_case_file(text).expect_err("a case with no verdict asserts nothing");
    assert!(error.contains("verdict"), "error names the field: {error}");
}

#[test]
fn an_oracle_step_missing_register_entry_is_rejected() {
    let text = r#"
[[case]]
name = "c"
kind = "oracle"
program = "r13.js"
verdict = "silent"
"#;
    let error = parse_case_file(text).expect_err("an unattributed verdict cannot regenerate §0.2");
    assert!(
        error.contains("register_entry"),
        "error names the field: {error}"
    );
}
