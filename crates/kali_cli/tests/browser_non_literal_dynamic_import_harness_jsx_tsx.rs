//! Task 18 batch 6A design-spec 5.11 retention: kept 100% hand-written, not
//! migrated. No case file exists for this target.
//!
//! WHAT BLOCKS IT. Both of this file's `#[test]` fns --
//! `run_rejects_non_literal_dynamic_import_targets_in_browser_api_surface_with_harness_jsx_and_tsx_input`
//! (`:165`) and
//! `test_rejects_non_literal_dynamic_import_targets_in_browser_api_surface_with_harness_jsx_and_tsx_input`
//! (`:184`) -- route through
//! `assert_browser_harness_rejects_non_literal_dynamic_import` (`:124`), and
//! each calls it twice per extension inside its own `for extension in [...]` loop,
//! once with the JSON-output flag false and once true. The true call is
//! unconditional, so it reaches `assert_non_literal_dynamic_import_rejection_json`
//! (`:101`, called at `:157`) on every iteration: 2 of 2 tests
//! reach the blocking construct and U4's trim-and-keep degenerates to whole-file
//! retention, because there is no complementary migratable subset to split off.
//!
//! The blocking construct is a PAIR OF QUANTIFIERS over the JSON `errors` array, at
//! `errors.iter().all(...)` (`:104`) and `errors.iter().any(...)` (`:108`). The
//! case-file format offers only closed dotted-path indexing into JSON -- design
//! spec 5.4 is explicit that there are "no slices, no wildcards, no
//! negative-from-end indexing, no filters" -- so a dotted path can pin the FIRST
//! array element and nothing more. Narrowing "every error has this code" to
//! "error 0 has this code" is a weakening (a second, differently-coded diagnostic
//! would satisfy the migration and fail the source), and rule 1 forbids weakening;
//! the existential is the mirror-image gap. Nothing in the twelve assertion keys
//! expresses either. The neighbouring `assert!(!errors.is_empty(), ...)`
//! (`:102`) is not the problem and would migrate on its own.
//!
//! ADJUDICATED, NOT PROPOSED. The human partner has ruled these quantifiers design
//! spec 5.11 outliers, in the same class as the position-anchored and
//! line-oriented sites 5.4's closing paragraph already places outside the
//! vocabulary: **no assertion key is being added for them.** The identical shape is
//! recorded in batch 3's `browser_generator_default_export_rejection.rs`, batch 5's
//! `browser_math_pow_optional_chain_harness.rs`, and batch 6A's trimmed
//! `browser_math_unsupported_member_calls_harness_jsx_tsx.rs`. Do not reopen this
//! by proposing a wildcard dotted path or a quantified-array key.
//!
//! The non-JSON half of the file WOULD have migrated cleanly --
//! `assert_non_literal_dynamic_import_rejection_text` (`:92`) is a
//! `stderr.contains("E5506")` plus a two-way OR that rule 11 resolves against the
//! real binary -- but a source `#[test]` fn cannot be split across the two halves:
//! each fn's own loop body runs both, on every iteration. U4's trim-and-keep unit
//! is the `#[test]` fn, so the expressible non-JSON arm yields no split. It was
//! re-derived mechanically for this batch rather than assumed, and the answer is
//! 2 / 2.
//!
//! CONSEQUENCE FOR THE GATES (ruling 9): THIS FILE HAS NO RED-LIST, and that is the
//! finding, not an omission. Ruling 9 addresses a U4 trim-and-keep retention, where
//! the on-disk `.rs` is shorter than the source its case file was migrated from.
//! Nothing was trimmed here, so there is no pre-trim/post-trim divergence and no
//! pre-trim ref to run anything against; and there is no right-hand side, since
//! `verify_pair.sh non_literal_dynamic_import_harness_jsx_tsx` exits 2 with a
//! missing case file before running any gate. Every gate that takes a `.rs`/`.toml`
//! pair therefore cannot run here at all. The ONE exception is the citation gate:
//! `batch5_crosscheck.py` needs no case file -- it resolves THIS header's own `:N`
//! citations against this very file. So a whole-file retention is not ungated: run
//! it directly, as
//! `batch5_crosscheck.py --citations-only non_literal_dynamic_import_harness_jsx_tsx`,
//! because `verify_pair.sh` still exits 2 before reaching it. It exits 0 today,
//! verified by running it rather than assumed. Ruling 11 exempts `:N` from the
//! no-moving-numbers rule only because it is mechanically gated, and this is where
//! that gating applies to a file with no pair.
//!
//! Batch 6A's one-line fix to that gate matters specifically here: until this batch,
//! `_needles` dropped method names shorter than four characters, so
//! `errors.iter().all(...)` and `errors.iter().any(...)` both reduced to
//! ['errors', 'iter'] and a citation onto either of the two lines above resolved
//! against the other. The two citations in this header are the reason the fix
//! landed first, alone, before any of this batch's case files were written.
//!
//! Escalated per rule 3/4 rather than shipped with a false green or a fabricated
//! claim. This file must NOT be deleted by the family-wide sweep after batch 8.
//! See the batch's own working report -- which was git-ignored scratch and
//! does not ship, so it is deliberately not cited by path.
use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn non_literal_dynamic_import_source() -> &'static str {
    "let specifier; import(specifier);"
}

fn non_literal_dynamic_import_test_source() -> &'static str {
    "Kali.test('dynamic import', () => { let specifier; return import(specifier); });\n"
}

fn assert_non_literal_dynamic_import_rejection_text(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("non-literal dynamic import()")
            || stderr.contains("statically known import specifier"),
        "stderr: {stderr}"
    );
}

fn assert_non_literal_dynamic_import_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains("non-literal dynamic import()")
            || error["message"]
                .as_str()
                .expect("error message")
                .contains("statically known import specifier")),
        "missing non-literal dynamic import in {errors:?}"
    );
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn assert_browser_harness_rejects_non_literal_dynamic_import(
    command: &str,
    filename: &str,
    source: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));

    if json_output {
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        let errors = json["errors"].as_array().expect("errors array");
        assert_non_literal_dynamic_import_rejection_json(errors);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_non_literal_dynamic_import_rejection_text(&stderr);
    }
}

#[test]
fn run_rejects_non_literal_dynamic_import_targets_in_browser_api_surface_with_harness_jsx_and_tsx_input(
) {
    for extension in ["jsx", "tsx"] {
        assert_browser_harness_rejects_non_literal_dynamic_import(
            "run",
            &format!("main.{extension}"),
            non_literal_dynamic_import_source(),
            false,
        );
        assert_browser_harness_rejects_non_literal_dynamic_import(
            "run",
            &format!("main.{extension}"),
            non_literal_dynamic_import_source(),
            true,
        );
    }
}

#[test]
fn test_rejects_non_literal_dynamic_import_targets_in_browser_api_surface_with_harness_jsx_and_tsx_input(
) {
    for extension in ["jsx", "tsx"] {
        assert_browser_harness_rejects_non_literal_dynamic_import(
            "test",
            &format!("smoke.test.{extension}"),
            non_literal_dynamic_import_test_source(),
            false,
        );
        assert_browser_harness_rejects_non_literal_dynamic_import(
            "test",
            &format!("smoke.test.{extension}"),
            non_literal_dynamic_import_test_source(),
            true,
        );
    }
}
