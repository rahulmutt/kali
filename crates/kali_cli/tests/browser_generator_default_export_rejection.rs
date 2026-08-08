//! Task 18 batch 3 escalation: kept 100% hand-written, not migrated. No case
//! file exists for this target.
//!
//! ALL 28 `#[test]` fns in this file reach
//! `assert_browser_harness_generator_rejection_with_expected_messages`
//! (`:162-226`) -- 16 of them indirectly, through the four-branch dispatcher
//! `assert_browser_harness_generator_rejection` (`:134-160`), and 12 directly
//! -- so U4's
//! trim-and-keep degenerates to whole-file retention: there is no complementary
//! migratable subset to split off. (Batch 3's two OTHER escalations,
//! `browser_math_abs_sign_frozen_aliases.rs` and
//! `browser_math_atan2_global_this_root.rs`, WERE trimmed to their one blocked
//! test each; this one could not be, because there is no unblocked subset.)
//!
//! TWO INDEPENDENT GROUNDS, EITHER OF WHICH ALONE RETAINS THIS FILE. They are
//! unrelated -- one is a tool blind spot, the other a format gap -- and they do
//! not overlap in their fix. **Removing one does not make this file
//! migratable.** A later reader who teaches `audit-case-migration.py` to see
//! fixture self-inspection, or who adds an assertion key for quantified JSON
//! arrays, must satisfy BOTH before reopening this target:
//!
//! (1) FIXTURE SELF-INSPECTION (audit blind spot; reaches 16 of the 28 fns).
//!     `assert_browser_harness_generator_rejection` (`:134-160`) selects its
//!     `expected_messages` by reading the JS fixture's OWN TEXT at `:142`,
//!     `:144` and `:146` (the blocking construct is the `if`/`else if` chain
//!     `:142-150`), before any command is built:
//!
//!     ```text
//!     if source.contains("(0, async function*") && matches!(command, "check" | "build") { ... }
//!     else if source.contains("yield*") { ... }
//!     else if source.contains("async function*") { ... }
//!     ```
//!
//!     `scripts/audit-case-migration.py` extracts each of those three
//!     `.contains` arguments as a claim, and `[source]` is excluded from its
//!     search by construction, so a migration reports them absent no matter
//!     what it contains -- they are read, never asserted on output. Verified,
//!     not assumed: a throwaway draft `.toml` carrying the strongest assertions
//!     the format can express for this file was audited against it and reported
//!     `AUDIT FAILED`, naming `(0, async function*` and `async function*` (the
//!     third needle, `yield*`, happens to survive only because it is a
//!     substring of the asserted message text `yield* delegation`). The
//!     controller has ruled the script is NOT extended for this shape
//!     (ruling 4).
//!
//! (2) UNIVERSALLY-QUANTIFIED JSON-ARRAY CLAIMS (format gap, spec 5.11; reaches
//!     all 28 fns). In `--output json` mode the shared helper asserts, at
//!     `:201`, `:202-205` and `:210-215`,
//!
//!     ```text
//!     assert!(!errors.is_empty(), "errors array should not be empty");
//!     assert!(errors.iter().all(|error| error["code"] == "E5506"), ...);
//!     assert!(expected_messages.iter().all(|expected| messages.iter().any(|m| m.contains(expected))), ...);
//!     ```
//!
//!     Both are universal quantifiers over the `errors` array. The case-file
//!     format offers only closed dotted-path indexing into JSON -- design spec
//!     5.4 is explicit that there are "no slices, no wildcards, no
//!     negative-from-end indexing, no filters" -- so `json.errors.0.code` can
//!     pin the FIRST error and nothing more. Narrowing "every error's code is
//!     E5506" to "error 0's code is E5506" is a weakening (a second,
//!     differently-coded diagnostic would satisfy the migration and fail the
//!     source), and rule 1 forbids weakening. Pinning `payload.errorCount` to
//!     restore the strength would add a claim the source never makes (rule 2).
//!     Every one of the 28 fns runs the json branch, because each loops
//!     `for json_output in [false, true]` inside its own body, so no fn is
//!     wholly free of this gap.
//!
//!     ADJUDICATED: the human partner ruled these quantifiers are §5.11
//!     outliers, in the same class as the `starts_with`/`lines()` sites that
//!     §5.4's closing paragraph already places outside the assertion
//!     vocabulary. **No eleventh assertion key is being added for them.** The
//!     disposition is retention, not a format extension, and it is settled --
//!     do not reopen it by proposing a `json.errors.*.code` wildcard or an
//!     `errors_all_code` key. Three further `browser_*` targets carry the same
//!     shape and fall in later batches
//!     (`browser_non_literal_dynamic_import_harness_jsx_tsx.rs`,
//!     `browser_math_pow_optional_chain_harness.rs`,
//!     `browser_math_unsupported_member_calls_harness_jsx_tsx.rs`); they are
//!     expected to be retained on this same ground.
//!
//! The non-json half of the file WOULD have migrated cleanly
//! (`exit = 1` for `assert_eq!(output.status.code(), Some(1))`, and
//! `stderr_contains = ["E5506", <expected message>]`), but a source `#[test]`
//! fn cannot be split across the two halves: each fn's own inner
//! `for json_output in [false, true]` loop runs both.
//!
//! Escalated per rule 3/4 rather than shipped with a false green or a
//! fabricated claim. This file must NOT be deleted by the family-wide sweep
//! after batch 8. See `.superpowers/sdd/2026-07-29-test-binary-consolidation/
//! task-18-batch3-report.md` for the full account.
//!
//! CONSEQUENCE FOR THE GATES (ruling 9), added retroactively by Task 18 batch 5:
//! THIS FILE HAS NO RED-LIST, and that is the finding, not an omission. Ruling 9
//! addresses a U4 trim-and-keep retention, where the on-disk `.rs` is shorter
//! than the source its case file was migrated from and every literal-comparison
//! gate therefore goes red against the wrong left-hand side. This is a
//! WHOLE-FILE retention -- the batch-3 commit that adjudicated it added this
//! header and deleted nothing -- so there is no pre-trim/post-trim divergence and
//! no pre-trim ref to run anything against. There is also no right-hand side:
//! `verify_pair.sh generator_default_export_rejection` exits 2 with
//! `missing .../cases/browser/generator_default_export_rejection.toml` before
//! running any gate, and every one of the five gates takes a `.rs`/`.toml` pair.
//! Verified by running it, not assumed. The batch-8 family gate's carve-out for
//! this file is the "must NOT be deleted" line above, not a gate red-list.
use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn generator_default_export_class_expression_source() -> &'static str {
    "export default (class NamedExample { *main() { yield* []; } });\n"
}

fn async_generator_default_export_class_expression_source() -> &'static str {
    "export default (class NamedExample { async *main() { yield* []; } });\n"
}

fn generator_default_export_sequence_wrapped_class_expression_source() -> &'static str {
    "export default (0, class NamedExample { *main() { yield* []; } });\n"
}

fn async_generator_default_export_sequence_wrapped_class_expression_source() -> &'static str {
    "export default (0, class NamedExample { async *main() { yield* []; } });\n"
}

fn generator_default_export_class_expression_no_delegate_source() -> &'static str {
    "export default (class NamedExample { *main() { yield 1; } });\n"
}

fn async_generator_default_export_class_expression_no_delegate_source() -> &'static str {
    "export default (class NamedExample { async *main() { yield 1; } });\n"
}

fn generator_default_export_sequence_wrapped_class_expression_no_delegate_source() -> &'static str {
    "export default (0, class NamedExample { *main() { yield 1; } });\n"
}

fn async_generator_default_export_sequence_wrapped_class_expression_no_delegate_source(
) -> &'static str {
    "export default (0, class NamedExample { async *main() { yield 1; } });\n"
}

fn assert_browser_harness_generator_rejection(
    command: &str,
    bundle: bool,
    extension: &str,
    source: &str,
    json_output: bool,
) {
    let expected_messages: &[&str] =
        if source.contains("(0, async function*") && matches!(command, "check" | "build") {
            &["generator and async-generator function lowering"]
        } else if source.contains("yield*") {
            &["yield* delegation"]
        } else if source.contains("async function*") {
            &["async-generator function lowering"]
        } else {
            &["generator function lowering"]
        };

    assert_browser_harness_generator_rejection_with_expected_messages(
        command,
        bundle,
        extension,
        source,
        json_output,
        expected_messages,
    );
}

fn assert_browser_harness_generator_rejection_with_expected_messages(
    command: &str,
    bundle: bool,
    extension: &str,
    source: &str,
    json_output: bool,
    expected_messages: &[&str],
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path())
        .env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node");
    if json_output {
        cli.arg("--output").arg("json");
    }
    cli.arg(command);
    if bundle {
        cli.arg("--bundle");
    }
    cli.arg("--api").arg("browser").arg(&source_path);

    let output = cli.output().expect("run kali");
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
        let messages = errors
            .iter()
            .map(|error| error["message"].as_str().expect("error message"))
            .collect::<Vec<_>>();
        assert!(
            expected_messages
                .iter()
                .all(|expected| messages.iter().any(|message| message.contains(expected))),
            "messages: {messages:?}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            expected_messages
                .iter()
                .all(|expected| stderr.contains(expected)),
            "stderr: {stderr}"
        );
    }
}

#[test]
fn check_rejects_anonymous_default_export_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "check",
                false,
                extension,
                "export default function*() { yield* []; }\n",
                json_output,
            );
        }
    }
}

#[test]
fn check_rejects_sequence_wrapped_anonymous_default_export_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "check",
                false,
                extension,
                "export default (0, function*() { yield* []; });\n",
                json_output,
            );
        }
    }
}

#[test]
fn run_rejects_anonymous_default_export_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "run",
                false,
                extension,
                "export default function*() { yield* []; }\n",
                json_output,
            );
        }
    }
}

#[test]
fn run_rejects_sequence_wrapped_anonymous_default_export_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "run",
                false,
                extension,
                "export default (0, function*() { yield* []; });\n",
                json_output,
            );
        }
    }
}

#[test]
fn check_rejects_anonymous_default_export_async_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "check",
                false,
                extension,
                "export default async function*() { yield* []; }\n",
                json_output,
            );
        }
    }
}

#[test]
fn check_rejects_sequence_wrapped_anonymous_default_export_async_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "check",
                false,
                extension,
                "export default (0, async function*() { yield* []; });\n",
                json_output,
            );
        }
    }
}

#[test]
fn run_rejects_anonymous_default_export_async_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "run",
                false,
                extension,
                "export default async function*() { yield* []; }\n",
                json_output,
            );
        }
    }
}

#[test]
fn run_rejects_sequence_wrapped_anonymous_default_export_async_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "run",
                false,
                extension,
                "export default (0, async function*() { yield* []; });\n",
                json_output,
            );
        }
    }
}

#[test]
fn test_rejects_anonymous_default_export_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "test",
                false,
                extension,
                "export default function*() { yield* []; }\n",
                json_output,
            );
        }
    }
}

#[test]
fn test_rejects_sequence_wrapped_anonymous_default_export_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "test",
                false,
                extension,
                "export default (0, function*() { yield* []; });\n",
                json_output,
            );
        }
    }
}

#[test]
fn test_rejects_anonymous_default_export_async_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "test",
                false,
                extension,
                "export default async function*() { yield* []; }\n",
                json_output,
            );
        }
    }
}

#[test]
fn test_rejects_sequence_wrapped_anonymous_default_export_async_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "test",
                false,
                extension,
                "export default (0, async function*() { yield* []; });\n",
                json_output,
            );
        }
    }
}

#[test]
fn build_rejects_anonymous_default_export_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "build",
                true,
                extension,
                "export default function*() { yield* []; }\n",
                json_output,
            );
        }
    }
}

#[test]
fn build_rejects_sequence_wrapped_anonymous_default_export_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "build",
                true,
                extension,
                "export default (0, function*() { yield* []; });\n",
                json_output,
            );
        }
    }
}

#[test]
fn build_rejects_anonymous_default_export_async_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "build",
                true,
                extension,
                "export default async function*() { yield* []; }\n",
                json_output,
            );
        }
    }
}

#[test]
fn build_rejects_sequence_wrapped_anonymous_default_export_async_generator_function_declarations_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_generator_rejection(
                "build",
                true,
                extension,
                "export default (0, async function*() { yield* []; });\n",
                json_output,
            );
        }
    }
}

#[test]
fn check_rejects_default_export_generator_and_async_generator_class_expressions_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_class_expression_source(),
                    &["generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
                (
                    async_generator_default_export_class_expression_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "check",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn run_rejects_default_export_generator_and_async_generator_class_expressions_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_class_expression_source(),
                    &["generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
                (
                    async_generator_default_export_class_expression_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "run",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn test_rejects_default_export_generator_and_async_generator_class_expressions_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_class_expression_source(),
                    &["generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
                (
                    async_generator_default_export_class_expression_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "test",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn build_rejects_default_export_generator_and_async_generator_class_expressions_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_class_expression_source(),
                    &["generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
                (
                    async_generator_default_export_class_expression_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "build",
                    true,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn check_rejects_sequence_wrapped_default_export_generator_and_async_generator_class_expressions_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_sequence_wrapped_class_expression_source(),
                    &["generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
                (
                    async_generator_default_export_sequence_wrapped_class_expression_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "check",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn run_rejects_sequence_wrapped_default_export_generator_and_async_generator_class_expressions_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_sequence_wrapped_class_expression_source(),
                    &["generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
                (
                    async_generator_default_export_sequence_wrapped_class_expression_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "run",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn test_rejects_sequence_wrapped_default_export_generator_and_async_generator_class_expressions_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_sequence_wrapped_class_expression_source(),
                    &["generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
                (
                    async_generator_default_export_sequence_wrapped_class_expression_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "test",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn build_rejects_sequence_wrapped_default_export_generator_and_async_generator_class_expressions_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_sequence_wrapped_class_expression_source(),
                    &["generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
                (
                    async_generator_default_export_sequence_wrapped_class_expression_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "build",
                    true,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn check_rejects_default_export_generator_and_async_generator_class_expressions_without_yield_delegation_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_class_expression_no_delegate_source(),
                    &["generator class method lowering is unavailable in the direct runtime path"][..],
                ),
                (
                    async_generator_default_export_class_expression_no_delegate_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "check",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn run_rejects_default_export_generator_and_async_generator_class_expressions_without_yield_delegation_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_class_expression_no_delegate_source(),
                    &["generator class method lowering is unavailable in the direct runtime path"][..],
                ),
                (
                    async_generator_default_export_class_expression_no_delegate_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "run",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn test_rejects_default_export_generator_and_async_generator_class_expressions_without_yield_delegation_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_class_expression_no_delegate_source(),
                    &["generator class method lowering is unavailable in the direct runtime path"][..],
                ),
                (
                    async_generator_default_export_class_expression_no_delegate_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "test",
                    false,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}

#[test]
fn build_rejects_default_export_generator_and_async_generator_class_expressions_without_yield_delegation_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        for json_output in [false, true] {
            for (source, expected_messages) in [
                (
                    generator_default_export_sequence_wrapped_class_expression_no_delegate_source(),
                    &["generator class method lowering is unavailable in the direct runtime path"][..],
                ),
                (
                    async_generator_default_export_sequence_wrapped_class_expression_no_delegate_source(),
                    &["async-generator class method lowering is unavailable in the direct runtime path"][..],
                ),
            ] {
                assert_browser_harness_generator_rejection_with_expected_messages(
                    "build",
                    true,
                    extension,
                    source,
                    json_output,
                    expected_messages,
                );
            }
        }
    }
}
