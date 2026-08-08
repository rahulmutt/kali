#!/usr/bin/env python3
"""Regression suite for `scripts/audit-case-migration.py`.

This is the migration gate for the test-binary-consolidation branch: it
proves that every literal claim a hand-written `.rs` test made survives into
the TOML case file that replaces it. Across Tasks 13-17, six separate bugs
were found in the gate itself -- every one discovered because a migration
happened to trip it, every one caught by a human/reviewer rather than by the
script. This suite pins each of those regressions with a minimal input that
reproduces the original defect, plus the two structural invariants
(key-sync with `model.rs`, and "excluded-by-construction" fields staying
excluded) whose violation has already happened twice.

stdlib `unittest` only -- no `pytest`, no third-party imports. The script
under test has a hyphen in its filename, so it is loaded by path via
`importlib.util.spec_from_file_location` rather than a normal `import`.

Run directly: `python3 scripts/audit-case-migration_test.py`
"""

from __future__ import annotations

import contextlib
import importlib.util
import io
import re
import sys
import tempfile
import unittest
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parent.parent
_SCRIPT_PATH = _REPO_ROOT / "scripts" / "audit-case-migration.py"
_MODEL_RS_PATH = _REPO_ROOT / "crates" / "kali_case_runner" / "src" / "model.rs"


def _load_audit_module():
    spec = importlib.util.spec_from_file_location("audit_case_migration", _SCRIPT_PATH)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


audit = _load_audit_module()


def _run_audit(old_source: str, new_toml_sources: dict) -> tuple:
    """Run `audit.main()` in-process against a temporary `old.rs` and one or
    more temporary `.toml` files (dict of filename -> contents), the same way
    `scripts/test-gate.sh`-adjacent callers invoke it as a subprocess, but
    in-process so failures point at the real traceback. Returns
    `(returncode, combined_stdout)`.
    """
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        old_path = tmp_path / "old.rs"
        old_path.write_text(old_source)
        new_paths = []
        for name, contents in new_toml_sources.items():
            p = tmp_path / name
            p.write_text(contents)
            new_paths.append(p)

        argv = sys.argv
        sys.argv = ["audit-case-migration.py", str(old_path)] + [str(p) for p in new_paths]
        buf = io.StringIO()
        try:
            with contextlib.redirect_stdout(buf):
                rc = audit.main()
        finally:
            sys.argv = argv
        return rc, buf.getvalue()


# ---------------------------------------------------------------------------
# Bug 1: JSON_KEY reading claims out of JS fixture text.
# ---------------------------------------------------------------------------
class Bug1_JsonKeyFromFixtureText(unittest.TestCase):
    """`\\[\\s*"(\\w+)"\\s*\\]` applied to whole `.rs` text matched
    `globalThis["String"]["fromCharCode"](65)` inside a raw-string fixture,
    minting phantom claims ("String", "fromCharCode") that only a fabricated
    assertion could satisfy. Fixed by blanking raw-string spans
    (`_blank_raw_strings`) before the `JSON_KEY` scan runs."""

    def test_fixture_only_subscript_is_not_a_claim(self):
        source = (
            'fn fixture() -> &\'static str {\n'
            '    r#"globalThis["String"]["fromCharCode"](65)"#\n'
            '}\n'
        )
        result = audit.claims(source)
        self.assertNotIn("String", result["json keys"])
        self.assertNotIn("fromCharCode", result["json keys"])

    def test_genuine_assert_eq_json_index_is_a_claim(self):
        source = (
            '#[test]\n'
            'fn real_claim() {\n'
            '    assert_eq!(json["errors"][0]["code"], "E5506");\n'
            '}\n'
        )
        result = audit.claims(source)
        self.assertIn("errors", result["json keys"])
        self.assertIn("code", result["json keys"])


# ---------------------------------------------------------------------------
# Bug 2: _RAW_STRING missing its left anchor.
# ---------------------------------------------------------------------------
class Bug2_RawStringLeftAnchor(unittest.TestCase):
    """Without `(?<![A-Za-z0-9_])`, `_RAW_STRING` fired on the trailing `r`
    *inside* a plain string literal ("...operato" + "r\"" read as a raw
    string open), so the span from `operator"` to the next real `"` --
    including a genuine `json["errors"][0]["code"]` -- was blanked and
    silently dropped (measured at 93 real keys across 92 files). Pin both
    directions: the false match must not fire, and a genuine raw string must
    still be masked (so a JS/TS fixture in a raw string doesn't leak phantom
    JSON-key claims, i.e. bug 1 above)."""

    def test_word_ending_in_r_before_quote_is_not_a_raw_string_open(self):
        source = (
            'assert!(stderr.contains("unsupported operator")); '
            'assert_eq!(json["errors"][0]["code"], "E5506");'
        )
        result = audit.claims(source)
        self.assertIn("errors", result["json keys"])
        self.assertIn("code", result["json keys"])

    def test_genuine_raw_string_at_a_token_boundary_is_still_masked(self):
        # `r#"..."#` immediately after `= ` (a real token start) must still
        # be recognized and masked -- the anchor fix must not regress the
        # thing it exists to do (see Bug 1).
        source = 'let s = r#"a["b"]"#;'
        result = audit.claims(source)
        self.assertEqual(dict(result["json keys"]), {})


# ---------------------------------------------------------------------------
# Bug 3: unquote() not decoding \u{XXXX}.
# ---------------------------------------------------------------------------
class Bug3_UnquoteUnicodeEscape(unittest.TestCase):
    """`"6\\nh\\u{e9}llo"` must canonicalize to the real UTF-8 text
    `6\\nhéllo`, not the six literal characters `\\u{e9}`, or a correct
    migrated `stdout = "6\\nhéllo\\n"` could never match it."""

    def test_unicode_escape_decodes_to_real_character(self):
        token = r'"6\nh\u{e9}llo"'
        self.assertEqual(audit.unquote(token), "6\nhéllo")

    def test_double_backslash_before_u_is_not_decoded(self):
        # `"\\u{e9}"` (escaped backslash, then literal text `u{e9}`) must NOT
        # be decoded as the real unicode escape -- see unquote()'s own doc
        # comment on why the `(?<!\\)` guard must run before the `\\\\` ->
        # `\\` collapse.
        token = r'"\\u{e9}"'
        self.assertEqual(audit.unquote(token), r'\u{e9}')


# ---------------------------------------------------------------------------
# Bug 4: unquote() and the `\`+newline string continuation.
# ---------------------------------------------------------------------------
class Bug4_UnquoteLineContinuation(unittest.TestCase):
    """Rust's `\\`+newline continuation drops the backslash, the newline,
    and all following leading whitespace (` \\t\\r\\n`) up to the next
    non-whitespace byte. An escaped `\\\\` immediately before a real newline
    is NOT a continuation -- that newline is a literal embedded newline."""

    def test_continuation_eats_backslash_newline_and_leading_whitespace(self):
        # Rust source: "foo\<newline>   bar" -> "foobar"
        token = '"foo\\' + '\n' + '   bar"'
        self.assertEqual(audit.unquote(token), "foobar")

    def test_continuation_eats_tabs_and_carriage_returns_too(self):
        token = '"foo\\' + '\n' + ' \t\r\t bar"'
        self.assertEqual(audit.unquote(token), "foobar")

    def test_escaped_backslash_before_real_newline_is_not_a_continuation(self):
        # Rust source: "foo\\<newline>bar" -- `\\` is an escaped literal
        # backslash, and the newline right after it is a real embedded
        # newline in the string, not a continuation escape.
        token = '"foo\\\\' + '\n' + 'bar"'
        self.assertEqual(audit.unquote(token), "foo\\\nbar")


# ---------------------------------------------------------------------------
# Bug 5: ASSERT_EQ_VALUE_SECOND's first-argument skip.
# ---------------------------------------------------------------------------
class Bug5_AssertEqSecondArgument(unittest.TestCase):
    """The old `[^,]*` first-argument placeholder stopped at the first comma
    anywhere in the text, including one inside a nested call, so
    `assert_eq!(run_js(&src.replace("var x = 1;", "var x = 2;")), "v=100\\n")`
    registered the fixture text `"var x = 2;"` as a claim (a phantom -- it
    lives only in `[source]`-shaped fixture-construction text) and MISSED
    the real claim `"v=100\\n"`. `_assert_eq_literal_tokens`'s balanced-paren,
    string-aware argument splitter must resolve to the correct top-level
    second argument through: a nested call, a comma inside a string literal,
    and nested parens/brackets."""

    def test_nested_call_with_its_own_comma_does_not_confuse_the_split(self):
        source = (
            'assert_eq!(run_js(&src.replace("var x = 1;", "var x = 2;")), '
            '"v=100\\n");'
        )
        tokens = audit._assert_eq_literal_tokens(source)
        self.assertEqual(tokens, ['"v=100\\n"'])
        self.assertNotIn('"var x = 2;"', tokens)

    def test_comma_inside_a_string_literal_first_argument_is_not_a_split_point(self):
        source = 'assert_eq!(format!("x, y"), "expected\\n");'
        tokens = audit._assert_eq_literal_tokens(source)
        self.assertEqual(tokens, ['"expected\\n"'])

    def test_nested_brackets_in_first_argument_do_not_confuse_the_split(self):
        source = 'assert_eq!(map.get(&["a","b","c"]).unwrap(), "expected\\n");'
        tokens = audit._assert_eq_literal_tokens(source)
        self.assertEqual(tokens, ['"expected\\n"'])


# ---------------------------------------------------------------------------
# Bug 6: Char literals in the scanner.
# ---------------------------------------------------------------------------
class Bug6_CharLiteralsInScanner(unittest.TestCase):
    """`'('`, `')'`, `'{'` were treated as bare brackets, corrupting
    paren/bracket depth tracking, so
    `assert_eq!(s.replace('(', "x"), "expected\\n")` silently lost its
    second argument entirely. The fix (`_char_literal_end`) must also NOT
    mistake a Rust lifetime (`'a`, `'static`, `'_`, `&'a str`,
    `fn f<'a>(...)`) for a char literal -- a lifetime starts the same way
    (bare `'` + identifier char) but is never closed by a second `'`."""

    def test_paren_char_literal_does_not_corrupt_depth_tracking(self):
        source = 'assert_eq!(s.replace(\'(\', "x"), "expected\\n");'
        tokens = audit._assert_eq_literal_tokens(source)
        self.assertEqual(tokens, ['"expected\\n"'])

    def test_brace_and_close_paren_char_literals_also_work(self):
        for ch in ('(', ')', '{', '}', '[', ']'):
            with self.subTest(char=ch):
                source = f"assert_eq!(s.replace('{ch}', \"x\"), \"expected\\n\");"
                tokens = audit._assert_eq_literal_tokens(source)
                self.assertEqual(tokens, ['"expected\\n"'])

    def test_lifetimes_are_not_mistaken_for_char_literals(self):
        # `_char_literal_end` returns None (not a char literal) for every
        # lifetime shape -- the plausible-closing-quote window never finds
        # one, because a lifetime identifier is never followed by `'`.
        self.assertIsNone(audit._char_literal_end("'a str", 0))
        self.assertIsNone(audit._char_literal_end("'static", 0))
        self.assertIsNone(audit._char_literal_end("'_ str", 0))
        self.assertIsNone(audit._char_literal_end("&'a str", 1))
        self.assertIsNone(audit._char_literal_end("fn f<'a>(x: &'a str)", 5))

    def test_lifetimes_do_not_corrupt_a_real_assert_eq_scan(self):
        source = (
            "fn f<'a>(s: &'a str) -> &'a str {\n"
            "    assert_eq!(s.replace('(', \"x\"), \"expected\\n\");\n"
            "    s\n"
            "}\n"
        )
        tokens = audit._assert_eq_literal_tokens(source)
        self.assertEqual(tokens, ['"expected\\n"'])


# ---------------------------------------------------------------------------
# Bug 7: CONST and re.DOTALL.
# ---------------------------------------------------------------------------
class Bug7_ConstDotall(unittest.TestCase):
    """A `const NAME: &str = "a\\<newline>   b";` (continuation-wrapped rule
    literal) was invisible to `CONST` without `re.DOTALL`, because `.` does
    not match a literal newline otherwise -- not a false claim, a false
    ABSENCE: `rule constants: 0` when a real constant was declared."""

    def test_continuation_wrapped_const_is_found(self):
        source = 'const NAME: &str = "a\\' + '\n' + '   b";'
        matches = audit.CONST.findall(source)
        self.assertEqual(len(matches), 1)

    def test_continuation_wrapped_const_without_dotall_would_be_invisible(self):
        # Documents *why* the fix is needed: the same pattern compiled
        # without re.DOTALL (i.e. the pre-fix behavior) finds nothing at all
        # on a continuation-wrapped literal.
        no_dotall = re.compile(
            r'const\s+[A-Z0-9_]+\s*:\s*&str\s*=\s*\n?\s*(' + audit._STR_LITERAL + r')'
        )
        source = 'const NAME: &str = "a\\' + '\n' + '   b";'
        self.assertEqual(no_dotall.findall(source), [])


# ---------------------------------------------------------------------------
# Invariant 8: key-sync with crates/kali_case_runner/src/model.rs.
# ---------------------------------------------------------------------------
class Invariant8_ModelRsKeySync(unittest.TestCase):
    """`_STEP_LIST_KEYS`, `_STEP_SCALAR_KEYS`, and `_STEP_JSON_KEYS` must
    together cover every assertion-carrying field of `Step`/`RawStep` --
    stated in the script's own comment above `_STEP_LIST_KEYS`, and violated
    twice (`json_null`, `stderr` each shipped without being added, leaving a
    new key's claims silently unaudited). This test parses `model.rs` itself
    and fails if a field exists there that this audit script neither reads
    via the three tuples nor accounts for in a named, one-line-justified
    list -- so adding a field to `model.rs` forces a deliberate decision
    here, the same way `_CASE_NON_STEP_KEYS`/`BORING` force one in the
    script itself.

    Two categories of "accounted for" that are NOT the three tuples:

    - `_NO_CLAIM_FIELDS`: exactly the exclusion list named in this task's
      brief (name, rationale, ignore, kind, path, entry, body, matrix, exit,
      and everything under [source]) -- fields that carry no literal claim
      by construction, per the audit script's own module docstring.
    - `_OTHERWISE_AUDITED_FIELDS`: fields that DO carry a claim (or gate
      structure) but are read by a separate, explicit code path rather than
      by membership in one of the three tuples: `env` (read directly in
      `_step_assertion_strings`'s `env = step.get("env")` block -- the task
      brief's exclusion list does not name it, which is a real gap between
      the brief and the code as it stands; see the report), `constants`
      (read directly in `assertion_strings`), and `case`/`step` (the
      containers `assertion_strings` iterates to reach every actual case/
      step -- structural, not claim fields in their own right).
    """

    # Fields with no literal claim, by construction -- see the audit
    # script's own module docstring for why each is safe to exclude.
    _NO_CLAIM_FIELDS = {
        "name": "case/file identifier, not an assertion",
        "rationale": "prose explaining a case, not an assertion",
        "ignore": "boolean control flag, not an assertion",
        "kind": "step-kind selector, not a literal value",
        "path": "file_json step's source path -- a file reference",
        "entry": "browser step's module entry point -- a file reference",
        "body": "browser step's inline JS body -- program text, like [source]",
        "matrix": "axis data; substituted into args/stdout_contains/etc. "
                  "before assertions are read, audited at the substitution site",
        "exit": "a real assertion (exit status) but not a string literal -- "
                "out of scope for a literal-coverage audit",
        "source": "program text ([source] fixtures), never an assertion",
    }

    # Fields that DO carry a claim (or are load-bearing structure) but are
    # read by explicit code in the audit script rather than by appearing in
    # one of the three key tuples.
    _OTHERWISE_AUDITED_FIELDS = {
        "env": "read directly in _step_assertion_strings via "
               "`env = step.get(\"env\")`, not via a _STEP_*_KEYS tuple",
        "constants": "read directly in assertion_strings via "
                     "`doc.get(\"constants\")`",
        "case": "container assertion_strings iterates (`doc.get(\"case\", [])`) "
                "to reach every actual case -- not a claim field itself",
        "step": "container assertion_strings iterates (`case.get(\"step\")`) "
                "to reach every [[case.step]] entry -- not a claim field itself",
    }

    # Not a real TOML key: `RawCase.rest` is a `#[serde(flatten)]` catch-all
    # whose *contents* are the inline step's own fields (already checked as
    # part of RawStep below). Excluded from consideration entirely, not
    # asserted about.
    _NOT_A_TOML_KEY = {"rest"}

    @staticmethod
    def _extract_struct_body(text: str, struct_name: str) -> str:
        """The text strictly between a Rust `struct NAME { ... }`'s outer
        braces, found by brace-depth counting (not a lazy-`.*?` regex) so a
        future field whose doc comment happens to contain `{`/`}` can't
        silently truncate the match."""
        marker = f"struct {struct_name} "
        start = text.find(marker)
        if start == -1:
            # Some structs in this file have no trailing space before `{`
            # variance (e.g. no derive fields) -- fall back to a looser find.
            marker = f"struct {struct_name}"
            start = text.find(marker)
        if start == -1:
            raise AssertionError(f"could not find `struct {struct_name}` in model.rs")
        open_brace = text.find("{", start)
        if open_brace == -1:
            raise AssertionError(f"could not find opening brace for struct {struct_name}")
        depth = 0
        i = open_brace
        n = len(text)
        while i < n:
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
                if depth == 0:
                    return text[open_brace + 1:i]
            i += 1
        raise AssertionError(f"unbalanced braces scanning struct {struct_name}")

    @classmethod
    def _field_names(cls, body: str) -> set:
        # A field declaration line looks like `    kind: Option<StepKind>,`
        # -- leading whitespace, an identifier, then a colon. Attribute
        # lines (`    #[serde(default)]`) don't match (they start with `#`,
        # not an identifier character), so this only picks up real fields.
        return set(re.findall(r'(?m)^\s*([a-zA-Z_][a-zA-Z0-9_]*):', body))

    def test_every_model_rs_field_is_read_by_the_audit_script_somewhere(self):
        text = _MODEL_RS_PATH.read_text()

        raw_step_fields = self._field_names(self._extract_struct_body(text, "RawStep"))
        raw_case_fields = self._field_names(self._extract_struct_body(text, "RawCase")) - self._NOT_A_TOML_KEY
        raw_case_file_fields = self._field_names(self._extract_struct_body(text, "RawCaseFile"))

        all_fields = raw_step_fields | raw_case_fields | raw_case_file_fields
        self.assertTrue(all_fields, "field extraction found nothing -- regex is broken")

        tuple_covered = (
            set(audit._STEP_LIST_KEYS) | set(audit._STEP_SCALAR_KEYS) | set(audit._STEP_JSON_KEYS)
        )
        accounted = tuple_covered | set(self._NO_CLAIM_FIELDS) | set(self._OTHERWISE_AUDITED_FIELDS)

        unaccounted = all_fields - accounted
        self.assertEqual(
            unaccounted,
            set(),
            f"model.rs field(s) {sorted(unaccounted)!r} are neither in one of "
            "_STEP_LIST_KEYS/_STEP_SCALAR_KEYS/_STEP_JSON_KEYS nor in this "
            "test's named exclusion lists -- a new assertion-carrying field "
            "was likely added to model.rs without teaching the audit script "
            "(or this test) to read it. This is exactly the json_null/stderr "
            "class of bug.",
        )

        # Guard the guard: every name in the exclusion lists must actually
        # exist somewhere in model.rs, or the lists are stale and silently
        # hiding nothing.
        for name in self._NO_CLAIM_FIELDS:
            if name == "source":
                continue  # top-level [source] table, not a struct field name collision risk
            self.assertIn(
                name, all_fields | {"source"},
                f"exclusion-list entry {name!r} does not correspond to any field in model.rs "
                "-- stale entry, remove it",
            )
        for name in self._OTHERWISE_AUDITED_FIELDS:
            self.assertIn(
                name, all_fields,
                f"exclusion-list entry {name!r} does not correspond to any field in model.rs "
                "-- stale entry, remove it",
            )


# ---------------------------------------------------------------------------
# Invariant 9: excluded-by-construction fields stay excluded.
# ---------------------------------------------------------------------------
class Invariant9_ExcludedFieldsStayExcluded(unittest.TestCase):
    """A claim-looking literal that appears ONLY in `name`, `rationale`, a
    `#` comment, `[source]`, or `body` must NOT satisfy an audit -- the
    module docstring names four concrete ways this previously produced a
    false "every claim present" (a `rationale = \"\"\"...\"\"\"` block, an
    inline `# ...` comment, a case `name`, and a `[source]` fixture's `//
    ...` comment), all four confirmed independently sufficient on their own
    to hide a wrong diagnostic code. This end-to-end test plants the SAME
    wrong-diagnostic scenario: the real assertion in the `.rs` file expects
    `E5507`; the case file's `json` field asserts the wrong `E5506`; and
    `E5507` is scattered across every excluded-by-construction home. The
    audit must still fail."""

    _OLD_SOURCE = (
        '#[test]\n'
        'fn regression_case() {\n'
        '    assert_eq!(json["errors"][0]["code"], "E5507");\n'
        '}\n'
    )

    _HIDDEN_TOML = '''\
[[case]]
name = "E5507 regression"
rationale = """
Ensures E5507 is reported by the CLI in this exact scenario.
"""
# E5507 comment, must not satisfy the audit
body = "/* E5507 */"
json = { errors = [ { code = "E5506" } ] }

[source]
fixture = "// E5507 fixture text, not a real assertion"
'''

    def test_wrong_code_hidden_in_excluded_homes_still_fails_the_audit(self):
        rc, out = _run_audit(self._OLD_SOURCE, {"new.toml": self._HIDDEN_TOML})
        self.assertEqual(rc, 1, out)
        self.assertIn("AUDIT FAILED", out)
        self.assertIn("E5507", out)

    def test_control_same_file_with_the_correct_code_passes(self):
        # Same file, `json`'s code corrected to E5507 -- proves the failure
        # above is really about the excluded homes, not an unrelated parse
        # error or a typo in the fixture.
        fixed = self._HIDDEN_TOML.replace('"E5506"', '"E5507"')
        rc, out = _run_audit(self._OLD_SOURCE, {"new.toml": fixed})
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)


# ---------------------------------------------------------------------------
# End-to-end: a minimal pair that passes, and one that must fail.
# ---------------------------------------------------------------------------
class EndToEndAudit(unittest.TestCase):
    _OLD_SOURCE = (
        '#[test]\n'
        'fn minimal_case() {\n'
        '    assert!(stdout.contains("hello world"));\n'
        '    assert_eq!(json["errors"][0]["code"], "E5506");\n'
        '}\n'
    )

    def test_matching_case_file_passes(self):
        toml_source = '''\
[[case]]
name = "minimal_case"
args = ["run"]
stdout_contains = ["hello world"]
json = { errors = [ { code = "E5506" } ] }
'''
        rc, out = _run_audit(self._OLD_SOURCE, {"new.toml": toml_source})
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)

    def test_dropped_claim_fails(self):
        # `stdout_contains` and `json.errors[0].code` were both altered --
        # a real migration dropping/weakening claims, not a typo in the
        # test itself.
        toml_source = '''\
[[case]]
name = "minimal_case"
args = ["run"]
stdout_contains = ["hello there"]
json = { errors = [ { code = "E5507" } ] }
'''
        rc, out = _run_audit(self._OLD_SOURCE, {"new.toml": toml_source})
        self.assertEqual(rc, 1, out)
        self.assertIn("AUDIT FAILED", out)
        self.assertIn("hello world", out)
        self.assertIn("E5506", out)


# ---------------------------------------------------------------------------
# Documented, accepted limitations -- pinned as CURRENT behavior, not as a
# desired property. A future change that accidentally alters any of these
# should show up here, not slip through silently.
# ---------------------------------------------------------------------------
class DocumentedLimitations(unittest.TestCase):
    def test_rprefix_inside_a_line_comment_is_still_read_as_a_raw_string_open(self):
        # `_RAW_STRING`'s own doc comment names this residual directly: the
        # scanner "cannot tell a genuine `r#\"` token start from the same
        # three characters appearing inside a line comment ... this is a
        # regex approximation, not a real Rust lexer." A `r#"` inside a `//`
        # comment, followed later by a `"#`-shaped closing fence, is
        # misread as a real raw-string span -- so real code (and a real
        # JSON-key claim) between them gets masked and silently dropped.
        # Documented as "not present anywhere in the corpus ... acceptable
        # for that reason, not because it is impossible in principle."
        source = (
            '// this mentions r#"weird\n'
            'fn f() {\n'
            '    assert_eq!(json["real_key"], "value");\n'
            '}\n'
            '"# more text\n'
        )
        result = audit.claims(source)
        # Current (accepted-limitation) behavior: the claim is lost. If this
        # is ever fixed, this assertion should flip to assertIn and this
        # comment should say so -- do not "fix" this test in isolation.
        self.assertNotIn("real_key", result["json keys"])

    def test_odd_backslash_run_before_a_unicode_escape_is_not_correctly_paired(self):
        # `_UNICODE_ESCAPE`'s `(?<!\\)` guard is a single-character
        # lookbehind, so it cannot do real odd/even backslash-run parity.
        # Rust semantics for source `"\\\u{e9}"` (three backslashes then
        # `u{e9}`) pair the first two backslashes into one literal `\` and
        # read the third backslash + `u{e9}` as the real escape `\u{e9}`
        # (i.e. the correct decode is one literal backslash followed by
        # `é`). The single-char lookbehind instead sees a backslash
        # immediately before the candidate and rejects it outright, so no
        # unicode decoding happens at all, and the final `\\` -> `\`
        # collapse only pairs off two of the three backslashes -- the
        # canonicalized text ends up as two literal backslashes followed by
        # the literal text `u{e9}`, not backslash+é. Not called out by name
        # in the script's comments (unlike the r#"-in-a-comment residual
        # above), but the same class of approximation, and worth pinning so
        # a change to the guard's shape is visible rather than silent.
        token = '"' + ('\\' * 3) + 'u{e9}' + '"'
        self.assertEqual(audit.unquote(token), '\\\\u{e9}')

    def test_byte_string_prefix_is_not_recognized_as_a_string_literal(self):
        # `_STR_LITERAL` is `r?#*"..."#*` -- there is no `b?`/`c?` prefix
        # handling anywhere in this script, so a hypothetical
        # `.contains(b"literal")` (a Rust byte-string literal) is not
        # matched at all. Not named in the script's comments and not
        # observed anywhere in this corpus at the time of writing (the
        # corpus only uses plain and raw string literals), but pinned here
        # as current behavior per the same "known gap, not silently
        # regressible" posture as the other two tests in this class.
        source = '.contains(b"literal")'
        self.assertEqual(audit.CONTAINS.findall(source), [])


if __name__ == "__main__":
    unittest.main()
