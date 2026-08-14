//! Task 18 batch 5 design-spec 5.11 retention: kept 100% hand-written, not
//! migrated. No case file exists for this target.
//!
//! WHAT BLOCKS IT. Both of this file's `#[test]` fns --
//! `build_rejects_optional_chain_wrapped_math_pow_in_browser_api_surface_with_js_ts_jsx_and_tsx_input`
//! (`:155`) and
//! `check_rejects_optional_chain_wrapped_math_pow_in_browser_api_surface_with_js_ts_jsx_and_tsx_input`
//! (`:174`) -- route through `assert_browser_math_pow_optional_chain_rejection`
//! (`:93`), and each calls it twice per extension inside its own
//! `for extension in [...]` loop, once with the JSON-output flag false and once
//! true. The true call is unconditional, so 2 of 2 tests reach the blocking
//! construct and U4's trim-and-keep degenerates to whole-file retention: there
//! is no complementary migratable subset to split off.
//!
//! The blocking construct is a pair of QUANTIFIERS over the JSON `errors` array,
//! at `errors.iter().all(...)` (`:135`) and `errors.iter().any(...)` (`:139`).
//! The case-file format offers
//! only closed dotted-path indexing into JSON -- design spec 5.4 is explicit that
//! there are "no slices, no wildcards, no negative-from-end indexing, no
//! filters" -- so a dotted path can pin the FIRST array element and nothing more.
//! Narrowing "every error has this code" to "error 0 has this code" is a
//! weakening (a second, differently-coded diagnostic would satisfy the migration
//! and fail the source), and rule 1 forbids weakening; the existential is the
//! mirror-image gap. Nothing in the twelve assertion keys expresses either.
//!
//! ADJUDICATED, NOT PROPOSED. The human partner has ruled these quantifiers
//! design-spec 5.11 outliers, in the same class as the position-anchored and
//! line-oriented sites 5.4's closing paragraph already places outside the
//! vocabulary: **no assertion key is being added for them.** The identical shape
//! is recorded in batch 3's `browser_generator_default_export_rejection.rs`,
//! which names this file as an expected later instance of it. Do not reopen this
//! by proposing a wildcard dotted path or a quantified-array key.
//!
//! The non-JSON half of the file WOULD have migrated cleanly, but a source
//! `#[test]` fn cannot be split across the two halves: each fn's own loop body
//! runs both.
//!
//! CONSEQUENCE FOR THE GATES (ruling 9): THIS FILE HAS NO RED-LIST, and that is
//! the finding, not an omission. Ruling 9 addresses a U4 trim-and-keep retention,
//! where the on-disk `.rs` is shorter than the source its case file was migrated
//! from. Nothing was trimmed here, so there is no pre-trim/post-trim divergence
//! and no pre-trim ref to run anything against; and there is no right-hand side,
//! since `verify_pair.sh math_pow_optional_chain_harness` exits 2 with a missing
//! case file before running any gate. FIVE of the six gates take a `.rs`/`.toml` pair and therefore cannot
//! run here at all. The SIXTH is the exception, and it changes this paragraph:
//! `batch5_crosscheck.py`, the citation gate that batch 6 wired into
//! `verify_pair.sh`, needs no case file -- it resolves THIS header's own `:N`
//! citations against this very file. So a whole-file retention is no longer
//! ungated: run it directly, as
//! `batch5_crosscheck.py --citations-only math_pow_optional_chain_harness`, because `verify_pair.sh`
//! still exits 2 before reaching it. It exits 0 today. Ruling 11 exempts `:N` from the
//! no-moving-numbers rule only because it is mechanically gated, and this is
//! where that gating applies to a file with no pair. Verified by running it, not assumed.
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

fn browser_math_pow_optional_chain_run_source() -> &'static str {
    r#"const exponent = 3;
const alias = exponent;
console.log(globalThis?.Math.pow(2, alias));
console.log(globalThis?.Math["pow"](2, alias));
console.log(globalThis?.Math['pow'](2, alias));
console.log(globalThis?.["Math"].pow(2, alias));
console.log(globalThis?.["Math"]["pow"](2, alias));
console.log(globalThis?.["Math"]['pow'](2, alias));
console.log(globalThis?.['Math'].pow(2, alias));
console.log(globalThis?.['Math']["pow"](2, alias));
console.log(globalThis?.['Math']['pow'](2, alias));
console.log(Object.freeze(globalThis?.Math.pow)(2, alias));
console.log(Object.freeze((globalThis?.Math.pow))(2, alias));
console.log(Object.freeze(globalThis?.Math["pow"])(2, alias));
console.log(Object.freeze((globalThis?.Math["pow"]))(2, alias));
console.log(Object.freeze((globalThis?.["Math"].pow))(2, alias));
console.log(Object.freeze((globalThis?.['Math'].pow))(2, alias));
console.log(Object.freeze((globalThis?.["Math"]["pow"]))(2, alias));
console.log(Object.freeze((globalThis?.["Math"]['pow']))(2, alias));
console.log(Object.freeze((globalThis?.['Math']["pow"]))(2, alias));
console.log(Object.freeze((globalThis?.['Math']['pow']))(2, alias));
"#
}

fn assert_browser_math_pow_optional_chain_rejection(
    command: &str,
    filename: &str,
    source: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path());
    if json_output {
        cli.arg("--output").arg("json");
    }
    let mut cli = cli.arg(command);
    if command == "build" {
        cli = cli.arg("--bundle");
    }
    let output = cli
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(1));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty(), "errors array should not be empty");
        assert!(
            errors.iter().all(|error| error["code"] == "E5506"),
            "unexpected errors: {errors:?}"
        );
        assert!(
            errors.iter().any(|error| error["message"]
                .as_str()
                .is_some_and(|message| message.contains("optional-chain wrappers"))),
            "unexpected errors: {errors:?}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            stderr.contains("optional-chain wrappers"),
            "stderr: {stderr}"
        );
    }
}

#[test]
fn build_rejects_optional_chain_wrapped_math_pow_in_browser_api_surface_with_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_math_pow_optional_chain_rejection(
            "build",
            &format!("app.{extension}"),
            browser_math_pow_optional_chain_run_source(),
            false,
        );
        assert_browser_math_pow_optional_chain_rejection(
            "build",
            &format!("app.{extension}"),
            browser_math_pow_optional_chain_run_source(),
            true,
        );
    }
}

#[test]
fn check_rejects_optional_chain_wrapped_math_pow_in_browser_api_surface_with_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_math_pow_optional_chain_rejection(
            "check",
            &format!("main.{extension}"),
            browser_math_pow_optional_chain_run_source(),
            false,
        );
        assert_browser_math_pow_optional_chain_rejection(
            "check",
            &format!("main.{extension}"),
            browser_math_pow_optional_chain_run_source(),
            true,
        );
    }
}
