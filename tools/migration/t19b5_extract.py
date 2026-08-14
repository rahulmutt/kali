#!/usr/bin/env python3
"""Forward extractor for Task 19 batch 5 -- the last migration batch.

WHAT IS DIFFERENT ABOUT THIS BATCH, AND WHY BATCH 4's EXTRACTOR DOES NOT FIT.
Batch 4's predicate was TEXT-CLI: no `.env(`, no `--output`, no `--bundle`/
`--api`, no `serde_json`. Every one of this batch's seven targets carries at
least one of those, which is exactly why they were left. So the claim vocabulary
is wider in a specific direction -- a claim is no longer only about two text
streams and an exit status, it can be about a JSON document parsed out of
stdout -- and a shape table closed over the old vocabulary would silently see
nothing on most of these files.

CLOSED OVER CLAIMS, NOT OVER ASSERTION MACROS. `residual_claims` blanks every
`assert*!` span, every permitted whole statement and every permitted call, then
REFUSES on any surviving claim token. That is the difference batch 3 had to add
in a fix round and batch 4 started from: a table keyed on `assert!` cannot see a
claim carried by `.expect()`, by `panic!`, or by plain control flow, and a
forward extractor that skips what it does not understand converts a dropped
claim into a green run.

THE CLAIM TABLE (closed; anything else raises `UnknownShape` naming the file,
the fn and the verbatim text):

  C1  assert!(<out>.status.success(), ..)                -> exit = "success"
  C2  assert!(!<out>.status.success(), ..)               -> exit = "failure"
  C3  assert_eq!(<out>.status.code(), Some(N))           -> exit = N
  C4  assert_eq!(<stdout-expr>, <str>)                   -> exact `stdout` pin
  C5  assert!(<stream-expr>.contains(<str>), ..)         -> stdout/stderr_contains
  C6  assert!(!<stream-expr>.contains(<str>), ..)        -> stdout/stderr_absent
  C7  assert!(!<out>.status.success()
              || <stdout-expr> == <str>, ..)             -> rule 11, resolved
  C8  assert!(<out>.status.success()
              && <stdout-expr> == <str>, ..)             -> exit + exact stdout
  C9  assert_eq!(<json-path>, <scalar literal>)          -> `json.<path>` pin
  C10 assert!(<json-path>.as_array().expect(..)
              .is_empty(), ..)                           -> `json.<path> = []`
  C11 assert_eq!(<json-path>, serde_json::json!([]))     -> `json.<path> = []`
  C12 assert!(<json-string-leaf>.contains(<str>), ..)    -> `json_count`, at_least 1
  C13 assert_eq!(<json-path>.as_str().expect(..)         -- reached through the
                 .contains(<str>), ..)                      same C12 path

C12 IS RULING 3's AMENDED CLAUSE 4 AND IT IS THE REASON THIS MODULE EXISTS.
When ruling 3 was written a `json` leaf had no substring form, so a plain
`.contains` against one was migrated as an exact pin. `json_count` arrived in
the batch-4 interlude and IS that substring form, so the amended rule binds
here: **plain `.contains(x)` against a `json` string leaf -> `json_count` with
`at_least = 1`; an exact `json.…` pin only where the source's own assertion is
exact.** Every C9/C11 pin below comes from an `assert_eq!` -- an exact source
assertion -- and every C12 comes from a `.contains`. The distinction is made by
the SOURCE's macro, never by what the binary happened to print.

`<str>` is a string literal, a `const`, or an identifier that resolves through
the evaluator's environment to one. Which stream an expression names is
resolved the same way, so a file that bound `stdout` to *stderr* would be read
correctly rather than by the variable's spelling.

FIXTURE TEXT IS COPIED, NEVER TYPED (rules 8/9). A fixture that exists as a
literal is taken from `lexer.find_string_literals` off the source's own bytes.
A fixture built by `format!`, or one level removed inside `kali_common::`, does
not exist as a literal anywhere and is taken from `t19b5_captures`, which holds
the byte-exact output of EXECUTING the real code -- never a hand-applied
substitution. `check_captured` in the generator re-checks each capture against
its own `.rs` before it is emitted.

WHAT THIS MODULE DELIBERATELY DOES NOT DO. It does not decide reachability, it
does not read `rationale` prose, and it does not look at any shipped case file.
It reads one `.rs` and reports what that `.rs` claims. Everything downstream --
which family a file lands in, what a header says, whether a gate is expected
red -- is the generator's job, so that a defect in the rendering cannot be
mistaken for a defect in the reading.

  Usage:
    t19b5_extract.py            # the census, per target, with its shapes
    t19b5_extract.py --list     # the work list (re-derived, ruling 13)
"""

from __future__ import annotations

import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "tools/task-18-browser-pilot"))
sys.path.insert(0, HERE)

from lexer import find_string_literals  # noqa: E402

# The LEXICAL layer is shared with batch 4 on purpose, and the SEMANTIC layer is
# not. `blank_strings`/`split_statements`/`_matching` are string-aware scanning
# over Rust text -- infrastructure with its own gates (batch 4's generator runs
# in `--gates-only`, so a regression there fails loudly and immediately). The
# evaluator, the claim table and the residue scan below are written fresh:
# those are what the probe's refusal arm exercises, and sharing them would make
# "a second extractor agreed" mean "the same code ran twice".
from t19b4_extract import (  # noqa: E402
    UnknownShape, blank_line_comments, blank_strings, _matching,
    split_top_commas, norm, split_statements, DirVal, PathVal, Source, prose,
)

TESTS = os.path.join(REPO, "crates/kali_cli/tests")

# ---------------------------------------------------------------------------
# The work list
# ---------------------------------------------------------------------------

# The seven targets, and the predicate that produces them is `select()` below,
# asserted against this list on every run (ruling 13). A corpus change that
# moves the work list fails the gate rather than quietly migrating a different
# set than the report describes.
STEMS = [
    "for_of_array_iteration_spread",
    "logical_assignment_wrapped_local_binding",
    "nullish_assignment_wrapped_local_binding",
    "parse_float_static_ascii",
    "runtime_forin",
    "thread_topology_json",
    "wrapped_call_targets_wrappers",
]

# The four unmigrated-CLEAN targets this batch DECLINES, with the ground. Each
# carries a `json_output: bool` parameter that every call site passes `false`,
# so the literals inside its `if json_output {…}` blocks are DEAD: values
# written in the source and asserted by no reachable path. Controller ruling R1
# (`progress.md:1644-1653`) sends that shape to a spec §5.11 retention and
# explicitly rules out both alternatives (a per-file audit exception; teaching
# the audit Rust reachability analysis). All four were adjudicated in Task 15
# and upheld on re-review; none carried a `//!` header until this batch, which
# is why they screen CLEAN and why a brief listed them as migratable.
#
# The measurement, not the citation, is what `check_declined` re-runs.
DECLINED = {
    "array_from_bracketed_root_wrappers",
    "array_from_fully_bracketed_single_quoted_wrappers",
    "array_from_global_this_dot_root_wrappers",
    "array_from_set_map_dot_root_aliases",
}

# `for_of_array_iteration_spread` is a U4 TRIM. This one `#[test]` asserts
# against the FIXTURE'S OWN TEXT (`source.matches(alias).count()`), never against
# a process, so it makes no claim a case file can carry and is invisible to
# `audit-case-migration.py` for the reason ruling 10 names. It stays
# hand-written; the other 34 migrate.
RETAINED = {
    "for_of_array_iteration_spread": {
        "browser_harness_test_wrapper_reuses_the_shared_array_from_inventory_"
        "in_both_loop_sections",
    },
}


# ---------------------------------------------------------------------------
# Values beyond batch 4's set
# ---------------------------------------------------------------------------

class Str:
    """A string, WITH ITS PROVENANCE.

    `origin` is load-bearing and is the mechanical form of rules 8 and 9. A
    fixture body may be `literal` (copied from the source's own bytes),
    `const`, or `capture` (the byte-exact output of executing the real code).
    It may NEVER be `format` -- a value this evaluator computed by applying
    Rust's substitution rules itself. `fs::write` enforces that, so
    hand-simulation cannot reach a case file even if someone later widens
    `format_value`.
    """
    __slots__ = ("value", "origin")

    def __init__(self, value: str, origin: str = "literal"):
        self.value = value
        self.origin = origin


class Opaque:
    """A value this extractor deliberately does not compute.

    Only ever legal as a directory NAME. `runtime_forin::run_source` builds one
    from a process id and an atomic counter; the runner gives every trial its own
    directory anyway, so the name is not a fact any case file carries. Any other
    use raises -- an opaque value reaching a fixture body or an assertion would
    put text into a case file that is not the source's text, which is a rule-9
    violation by construction.
    """
    __slots__ = ("why",)

    def __init__(self, why: str):
        self.why = why


class BoolVal:
    __slots__ = ("value",)

    def __init__(self, value: bool):
        self.value = value


class CmdVal:
    """A `Command` under construction.

    These sources build argv across several statements and two `if` arms
    (`cli.arg("--output").arg("json"); cli.arg(command); if browser_harness {
    cli.arg("--api")… }`), so the builder is a mutable value in the environment
    rather than one expression. `env` is carried the same way: `.env(K, V)` is a
    step field in the case format, not an argv token, and conflating the two is
    how a browser-harness case silently loses its harness.
    """

    def __init__(self):
        self.argv: list = []
        self.env: dict[str, str] = {}
        self.dirs: set = set()


class OutputVal:
    __slots__ = ("inv",)

    def __init__(self, inv):
        self.inv = inv


class StreamVal:
    """`String::from_utf8_lossy(&out.stdout)` -- one of the two text streams."""
    __slots__ = ("inv", "stream")

    def __init__(self, inv, stream: str):
        self.inv = inv
        self.stream = stream


class JsonVal:
    """`serde_json::from_slice(&out.stdout)` -- the whole document."""
    __slots__ = ("inv",)

    def __init__(self, inv):
        self.inv = inv


class JsonPath:
    """A dotted path into an invocation's JSON stdout.

    `path` is already in the case format's spelling (`payload.threadTopology.
    liveInstances`), built segment by segment as the source indexes, so an
    intermediate binding (`let payload = &json["payload"];`) and a fully
    spelled-out index chain produce the SAME path. That is what lets
    `thread_topology_json.rs`'s two styles land on one claim vocabulary.
    """
    __slots__ = ("inv", "path")

    def __init__(self, inv, path: str):
        self.inv = inv
        self.path = path


class JsonStr:
    """`json[...].as_str().expect(..)` -- a JSON STRING LEAF.

    Its own type, because it is the receiver ruling 3's amended clause 4 is
    about: a `.contains` against this is a `json_count`, never an exact pin.
    """
    __slots__ = ("inv", "path")

    def __init__(self, inv, path: str):
        self.inv = inv
        self.path = path


class Invocation:
    """One real `kali` process the source runs."""

    def __init__(self, fn_name: str, where: str):
        self.fn_name = fn_name
        self.where = where
        self.argv: list = []
        self.env: dict[str, str] = {}
        self.fixtures: dict[str, str] = {}
        self.claims: list[dict] = []
        self.dirs: set = set()
        self.order = 0

    def argv_tokens(self) -> list[str]:
        return [t.name if isinstance(t, PathVal) else t for t in self.argv]

    def add(self, claim: dict):
        # A duplicate claim is the same claim; the sources assert
        # `status.success()` and then `status.code() == Some(0)` on the same
        # invocation, and two identical `stdout_contains` entries would render
        # a case file asserting one thing twice.
        if claim not in self.claims:
            self.claims.append(claim)


# ---------------------------------------------------------------------------
# The evaluator
# ---------------------------------------------------------------------------

_IDENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")
_INT = re.compile(r"^-?\d+$")

# Command builder methods this evaluator models. Anything else on a `Command`
# raises -- an unmodelled builder method is an argv token or an environment
# variable the case file would not carry.
_CMD_METHODS = {"arg", "args", "env", "current_dir", "output"}


class Evaluator:
    def __init__(self, src: Source, captures):
        self.src = src
        self.cap = captures
        self.invocations: list[Invocation] = []
        self.reached: dict[str, set[str]] = {}   # helper fn -> case names
        self.dead: list[str] = []                # values computed, never claimed
        self._depth = 0

    # -- helpers ---------------------------------------------------------

    def fail(self, where: str, what: str, text: str):
        raise UnknownShape(f"{self.src.stem}.rs::{where}: {what}: "
                           f"{norm(text)[:160]!r}")

    def record_reach(self, fn: str, case: str):
        self.reached.setdefault(fn, set()).add(case)

    # -- expressions -----------------------------------------------------

    def ev(self, expr: str, env: dict, where: str):
        e = expr.strip()
        while e.startswith("(") and _matching(e, 0, "(", ")") == len(e):
            e = e[1:-1].strip()
        if e.startswith("&"):
            return self.ev(e[1:], env, where)

        lits = find_string_literals(e)
        if lits and lits[0]["start"] == 0 and lits[0]["end"] == len(e):
            return Str(lits[0]["value"])
        if _INT.match(e):
            return int(e)
        if e == "true":
            return BoolVal(True)
        if e == "false":
            return BoolVal(False)
        if e == "Some(0)":
            return ("Some", 0)
        m = re.fullmatch(r"Some\(\s*(-?\d+)\s*\)", e)
        if m:
            return ("Some", int(m.group(1)))

        if _IDENT.match(e):
            if e in env:
                return env[e]
            if e in self.src.consts:
                return Str(self.src.consts[e])
            self.fail(where, "unbound identifier", e)

        # `serde_json::json!([])` -- the only `json!` form in this batch, and it
        # is spelled out rather than parsed as JSON so a `json!({...})` raises
        # instead of being guessed at.
        if re.fullmatch(r"serde_json::json!\s*\(\s*\[\s*\]\s*\)", e):
            return ("json_literal", [])

        if e.startswith("format!"):
            v = self.format_value(e, env, where)
            return v if isinstance(v, Opaque) else Str(v, "format")

        # method / index chains
        return self.chain(e, env, where)

    def chain(self, e: str, env: dict, where: str):
        """Evaluate an index/method chain left to right."""
        # `Command::new(kali_bin())` opens a builder.
        m = re.match(r"Command::new\s*\(", e)
        if m:
            end = _matching(e, m.end() - 1, "(", ")")
            inner = e[m.end():end - 1].strip()
            if inner not in ("kali_bin()",):
                self.fail(where, "Command::new on something other than kali_bin()", e)
            cmd = CmdVal()
            return self.apply_methods(cmd, e[end:], env, where, e)

        m = re.match(r"tempdir\s*\(\s*\)", e)
        if m:
            return self.apply_methods(DirVal(), e[m.end():], env, where, e)

        m = re.match(r"std::env::temp_dir\s*\(\s*\)", e)
        if m:
            return self.apply_methods(DirVal(), e[m.end():], env, where, e)

        m = re.match(r"String::from_utf8_lossy\s*\(", e)
        if m:
            end = _matching(e, m.end() - 1, "(", ")")
            v = self.ev(e[m.end():end - 1], env, where)
            if not isinstance(v, StreamVal):
                self.fail(where, "from_utf8_lossy on a non-stream", e)
            return self.apply_methods(v, e[end:], env, where, e)

        m = re.match(r"serde_json::from_slice\s*\(", e)
        if m:
            end = _matching(e, m.end() - 1, "(", ")")
            v = self.ev(e[m.end():end - 1], env, where)
            if not isinstance(v, StreamVal) or v.stream != "stdout":
                self.fail(where, "from_slice on something other than stdout", e)
            return self.apply_methods(JsonVal(v.inv), e[end:], env, where, e)

        if re.match(r"^match\b", e):
            return self.match_expr(e, env, where)

        m = re.match(r"([A-Za-z_][A-Za-z0-9_]*)\s*\(", e)
        if m and m.group(1) in self.src.fns:
            end = _matching(e, m.end() - 1, "(", ")")
            args = split_top_commas(e[m.end():end - 1])
            v = self.call(m.group(1), args, env, where)
            return self.apply_methods(v, e[end:], env, where, e)

        m = re.match(r"([A-Za-z_][A-Za-z0-9_]*)", e)
        if m:
            head = m.group(1)
            if head in env:
                return self.apply_methods(env[head], e[m.end():], env, where, e)
            if head in self.src.consts:
                return self.apply_methods(Str(self.src.consts[head]),
                                          e[m.end():], env, where, e)
        self.fail(where, "expression outside the modelled language", e)

    def match_expr(self, e: str, env: dict, where: str):
        """`match <scalar> { "lit" => <expr>, _ => <expr> }`.

        The scrutinee must be constant at the call site, exactly as an `if`
        condition must be -- a `match` whose arm cannot be decided statically
        would fan a case into combinations the source never ran (rule 2).
        """
        brace = e.index("{")
        scrut = self.ev(e[5:brace], env, where)
        if not isinstance(scrut, Str):
            self.fail(where, "`match` on something that is not a literal here", e)
        end = _matching(e, brace, "{", "}")
        if e[end:].strip():
            self.fail(where, "a `match` with a trailing chain is not modelled", e)
        default = None
        for arm in split_top_commas(e[brace + 1:end - 1]):
            if "=>" not in arm:
                self.fail(where, "unmodelled `match` arm", arm)
            pat, body = arm.split("=>", 1)
            pat = pat.strip()
            if pat == "_":
                default = body
                continue
            lits = find_string_literals(pat)
            if not (lits and lits[0]["start"] == 0 and lits[0]["end"] == len(pat)):
                self.fail(where, "a `match` pattern that is not a literal or `_`", arm)
            if lits[0]["value"] == scrut.value:
                return self.ev(body, env, where)
        if default is None:
            self.fail(where, "a `match` with no arm taken and no `_`", e)
        return self.ev(default, env, where)

    def apply_methods(self, val, rest: str, env: dict, where: str, whole: str):
        rest = rest.strip()
        while rest:
            if rest.startswith("["):
                end = _matching(rest, 0, "[", "]")
                key = self.ev(rest[1:end - 1], env, where)
                val = self.index(val, key, where, whole)
                rest = rest[end:].strip()
                continue
            m = re.match(r"\.\s*([A-Za-z_][A-Za-z0-9_]*)\s*", rest)
            if not m:
                self.fail(where, "unmodelled chain segment", rest)
            name = m.group(1)
            after = rest[m.end():]
            if after.startswith("("):
                end = _matching(after, 0, "(", ")")
                args = split_top_commas(after[1:end - 1])
                rest = after[end:].strip()
            else:
                # a FIELD access (`out.stdout`, `out.status`), not a call
                args = None
                rest = after.strip()
            val = self.member(val, name, args, env, where, whole)
        return val

    def index(self, val, key, where: str, whole: str):
        if isinstance(val, (JsonVal, JsonPath)):
            base = "" if isinstance(val, JsonVal) else val.path
            seg = key.value if isinstance(key, Str) else (
                str(key) if isinstance(key, int) else None)
            if seg is None:
                self.fail(where, "json index that is not a literal", whole)
            return JsonPath(val.inv, f"{base}.{seg}" if base else seg)
        self.fail(where, "index into an unmodelled value", whole)

    def member(self, val, name: str, args, env: dict, where: str, whole: str):
        # ---- Command builder
        if isinstance(val, CmdVal):
            if name not in _CMD_METHODS:
                self.fail(where, f"unmodelled Command method `.{name}`", whole)
            if name == "arg":
                val.argv.append(self.argv_token(args[0], env, where, whole))
                return val
            if name == "args":
                self.fail(where, "`.args(...)` is not modelled", whole)
            if name == "env":
                k = self.env_key(args[0], where, whole)
                v = self.ev(args[1], env, where)
                if not isinstance(v, Str):
                    self.fail(where, "`.env` value is not a literal", whole)
                val.env[k] = v.value
                return val
            if name == "current_dir":
                d = self.ev(args[0], env, where)
                if isinstance(d, DirVal):
                    val.dirs.add(d.uid)
                    return val
                self.fail(where, "`.current_dir` on a non-directory", whole)
            if name == "output":
                return self.spawn(val, env, where)
        # ---- process Output
        if isinstance(val, OutputVal):
            if name == "expect":
                return val          # `.output().expect("run kali")`
            if name == "status":
                return ("status", val.inv)
            if name in ("stdout", "stderr"):
                return StreamVal(val.inv, name)
            self.fail(where, f"unmodelled Output member `.{name}`", whole)
        if isinstance(val, tuple) and val and val[0] == "status":
            if name == "success":
                return ("success", val[1])
            if name == "code":
                return ("code", val[1])
            self.fail(where, f"unmodelled ExitStatus member `.{name}`", whole)
        # ---- text streams
        if isinstance(val, StreamVal):
            if name == "contains":
                return ("contains", val, self.needle(args[0], env, where, whole))
            if name in ("clone", "to_string", "as_ref", "trim_end_matches"):
                self.fail(where, f"unmodelled stream adapter `.{name}`", whole)
            self.fail(where, f"unmodelled stream member `.{name}`", whole)
        # ---- json
        if isinstance(val, (JsonVal, JsonPath)):
            path = "" if isinstance(val, JsonVal) else val.path
            if name == "expect":
                return val          # `from_slice(..).expect("json stdout")`
            if name == "as_str":
                return JsonStr(val.inv, path)
            if name == "as_array":
                return ("as_array", val.inv, path)
            if name == "as_object":
                return ("as_object", val.inv, path)
            self.fail(where, f"unmodelled json member `.{name}`", whole)
        if isinstance(val, tuple) and val and val[0] in ("as_array", "as_object"):
            if name == "expect":
                return val
            if name == "is_empty":
                return ("is_empty", val[1], val[2], val[0])
            self.fail(where, f"unmodelled `.{name}` on {val[0]}", whole)
        if isinstance(val, JsonStr):
            if name == "expect":
                return val
            if name == "contains":
                return ("json_contains", val,
                        self.needle(args[0], env, where, whole))
            self.fail(where, f"unmodelled json-string member `.{name}`", whole)
        # ---- directories and paths
        if isinstance(val, DirVal):
            if name == "path":
                return val
            if name == "join":
                nm = self.ev(args[0], env, where)
                if isinstance(nm, Opaque):
                    return DirVal()     # an opaque name can only name a dir
                if not isinstance(nm, Str):
                    self.fail(where, "`.join` on a non-literal name", whole)
                return PathVal(nm.value, val)
            if name == "expect":
                return val
            self.fail(where, f"unmodelled directory member `.{name}`", whole)
        if isinstance(val, PathVal):
            if name == "expect":
                return val
            self.fail(where, f"unmodelled path member `.{name}`", whole)
        if isinstance(val, Str):
            if name == "as_ref":
                return val
            if name == "matches":
                # The fixture-self-inspection shape (ruling 10). Reaching it
                # here means a `#[test]` this extractor was asked to migrate
                # asserts about the FIXTURE'S OWN TEXT rather than about a
                # process, which no case file can express. Refuse; the trim is
                # a decision for the generator's RETAINED list, not something
                # to be inferred silently at extraction time.
                self.fail(where,
                          "fixture self-inspection (`.matches` on fixture text) "
                          "-- ruling 10; this test cannot be migrated", whole)
            self.fail(where, f"unmodelled string member `.{name}`", whole)
        if val is None:
            self.fail(where, f"`.{name}` on nothing", whole)
        self.fail(where, f"unmodelled member `.{name}`", whole)

    # -- the pieces a Command is built from -------------------------------

    def argv_token(self, arg: str, env: dict, where: str, whole: str):
        v = self.ev(arg, env, where)
        if isinstance(v, Str):
            return v.value
        if isinstance(v, PathVal):
            return v
        self.fail(where, "argv token that is neither a literal nor a fixture path",
                  whole)

    def env_key(self, arg: str, where: str, whole: str) -> str:
        """The environment variable's NAME.

        Two spellings occur and they must resolve to the same string:
        `"KALI_BROWSER_BUNDLE_HARNESS_COMMAND"` written out, and
        `kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV`. The second is a
        cross-crate constant, so its value is READ OUT of that crate rather than
        assumed -- a rename there would otherwise silently keep emitting the old
        name into every case file.
        """
        a = arg.strip()
        lits = find_string_literals(a)
        if lits and lits[0]["start"] == 0 and lits[0]["end"] == len(a):
            return lits[0]["value"]
        if a.split("::")[-1] == "BROWSER_HARNESS_COMMAND_ENV":
            return browser_harness_env_name()
        self.fail(where, "`.env` key outside the modelled set", whole)

    def spawn(self, cmd: CmdVal, env: dict, where: str) -> OutputVal:
        inv = Invocation(env["__case__"], where)
        inv.argv = list(cmd.argv)
        inv.env = dict(cmd.env)
        inv.dirs = set(cmd.dirs)
        for tok in cmd.argv:
            if isinstance(tok, PathVal):
                inv.dirs.add(tok.dir.uid)
        inv.fixtures = dict(env.get("__fixtures__", {}))
        inv.order = len(self.invocations)
        self.invocations.append(inv)
        return OutputVal(inv)

    # -- format! ---------------------------------------------------------

    def format_value(self, e: str, env: dict, where: str) -> str:
        """A `format!` whose every hole resolves to a value already in hand.

        Rule 8 forbids hand-simulating a `format!`, and this is not that: it
        raises unless the template is a single literal with only `{ident}` holes
        naming values the evaluator already holds, and the FIXTURES it produces
        are never taken from here -- they come from `t19b5_captures`, which is
        the byte-exact output of executing the real code. What this path is for
        is argv (`format!("main.{extension}")`), where the value is a filename
        the case file spells out anyway.
        """
        start = e.index("(")
        end = _matching(e, start, "(", ")")
        args = split_top_commas(e[start + 1:end - 1])
        if not args:
            self.fail(where, "empty format!", e)
        lits = find_string_literals(args[0])
        if not (lits and lits[0]["start"] == 0 and
                lits[0]["end"] == len(args[0].strip())):
            self.fail(where, "format! template is not a single literal", e)
        tmpl = lits[0]["value"]
        if len(args) > 1:
            # A `format!` with positional arguments is not resolved. It is
            # returned OPAQUE, which is legal only as a directory name; every
            # other use raises. `runtime_forin::run_source` is the one site,
            # and what it builds is a temp-directory name the runner replaces.
            return Opaque("format! with positional arguments")
        out, i = [], 0
        while i < len(tmpl):
            c = tmpl[i]
            if c == "{" and i + 1 < len(tmpl) and tmpl[i + 1] == "{":
                out.append("{")
                i += 2
                continue
            if c == "}" and i + 1 < len(tmpl) and tmpl[i + 1] == "}":
                out.append("}")
                i += 2
                continue
            if c == "{":
                j = tmpl.index("}", i)
                name = tmpl[i + 1:j]
                if not _IDENT.match(name):
                    self.fail(where, "format! hole that is not a bare identifier", e)
                v = env.get(name)
                if v is None and name in self.src.consts:
                    v = Str(self.src.consts[name])
                if not isinstance(v, Str):
                    self.fail(where, f"format! hole `{name}` is not a string", e)
                out.append(v.value)
                i = j + 1
                continue
            out.append(c)
            i += 1
        return "".join(out)

    # -- statements ------------------------------------------------------

    def run_block(self, block: str, env: dict, where: str):
        self._depth += 1
        if self._depth > 40:
            raise UnknownShape(f"{where}: recursion bound reached")
        try:
            for st in split_statements(block):
                self.run_stmt(st, env, where)
        finally:
            self._depth -= 1

    def run_stmt(self, st: str, env: dict, where: str):
        s = st.strip().rstrip(";").strip()
        if not s:
            return

        # `for <var> in [ ... ] { ... }`
        m = re.match(r"^for\s+([A-Za-z_]\w*)\s+in\s*", s)
        if m:
            rest = s[m.end():].lstrip()
            if not rest.startswith("["):
                self.fail(where, "`for` over something other than an array literal", s)
            close = _matching(rest, 0, "[", "]")
            items = split_top_commas(rest[1:close - 1])
            body = rest[close:].lstrip()
            if not body.startswith("{"):
                self.fail(where, "`for` body is not a block", s)
            bend = _matching(body, 0, "{", "}")
            for it in items:
                child = dict(env)
                child[m.group(1)] = self.ev(it, env, where)
                self.run_block(body[:bend], child, where)
                env["__fixtures__"] = child.get("__fixtures__", env.get("__fixtures__", {}))
            return

        # `if <cond> { ... } [else { ... }]`
        if re.match(r"^if\b", s):
            head_end = s.index("{")
            cond = s[2:head_end].strip()
            bend = _matching(s, head_end, "{", "}")
            then = s[head_end:bend]
            tail = s[bend:].lstrip()
            els = None
            if tail.startswith("else"):
                t2 = tail[4:].lstrip()
                if t2.startswith("{"):
                    els = t2[:_matching(t2, 0, "{", "}")]
                elif t2.startswith("if"):
                    els = "{" + t2 + "}"
                else:
                    self.fail(where, "unmodelled `else` form", s)
            taken = self.cond(cond, env, where, s)
            if taken:
                self.run_block(then, env, where)
            elif els is not None:
                self.run_block(els, env, where)
            return

        # `let [mut] <name>[: T] = <expr>`
        m = re.match(r"^let\s+(?:mut\s+)?([A-Za-z_]\w*)\s*(?::[^=]+)?=\s*", s)
        if m:
            env[m.group(1)] = self.ev(s[m.end():], env, where)
            return

        # a bare `assert*!`
        if re.match(r"^assert(?:_eq|_ne)?!\s*\(", s):
            self.assertion(s, env, where)
            return

        # `fs::write(&path, body).expect(..)` -- a fixture
        m = re.match(r"^(?:std::)?fs::write\s*\(", s)
        if m:
            end = _matching(s, m.end() - 1, "(", ")")
            args = split_top_commas(s[m.end():end - 1])
            p = self.ev(args[0], env, where)
            body = self.ev(args[1], env, where)
            if not isinstance(p, PathVal):
                self.fail(where, "fs::write to a non-path", s)
            if not isinstance(body, Str):
                self.fail(where, "fs::write of a non-literal body", s)
            if body.origin == "format":
                # RULE 8, mechanically. A `format!` this evaluator resolved by
                # applying Rust's substitution rules is a hand-simulation, and
                # hand-derivation ships a DIFFERENT program that can still trip
                # the same diagnostic -- so the real-binary check would verify
                # the corrupted fixture against itself. The only admissible
                # value here is one that was executed.
                key = f"{self.src.stem}::{env['__case__']}"
                cap = self.captured(key)
                if cap is None:
                    self.fail(where,
                              f"a `format!`-built fixture with no capture -- "
                              f"run t19b5_capture_run.py and register {key!r}", s)
                if cap != body.value:
                    # The recomputation is a CROSS-CHECK, never the value. A
                    # disagreement means one of the two is wrong and neither may
                    # be preferred silently.
                    self.fail(where,
                              f"capture {key!r} disagrees with the structural "
                              f"recomputation of this `format!`", s)
                body = Str(cap, "capture")
            fx = dict(env.get("__fixtures__", {}))
            fx[p.name] = body.value
            env["__fixtures__"] = fx
            return

        if re.match(r"^(?:std::)?fs::create_dir_all\s*\(", s):
            return

        # A `static` item inside a fn body. `runtime_forin::run_source` declares
        # its temp-directory counter this way. It contributes to the directory
        # NAME and to nothing a case file carries -- the runner gives every trial
        # its own directory -- so it is skipped rather than modelled. It is
        # matched as a whole statement, so a `static` holding fixture text would
        # still have to reach `fs::write`, where its provenance is checked.
        if re.match(r"^static\s+[A-Z]", s):
            return

        # a bare statement-position expression: a builder mutation
        # (`cli.arg("run");`) or a call to a modelled helper.
        if re.match(r"^[A-Za-z_][\w:]*\s*[.(]", s):
            self.ev(s, env, where)
            return

        self.fail(where, "statement outside the modelled language", s)

    def cond(self, cond: str, env: dict, where: str, whole: str) -> bool:
        c = cond.strip()
        while c.startswith("(") and _matching(c, 0, "(", ")") == len(c):
            c = c[1:-1].strip()
        neg = False
        while c.startswith("!"):
            neg = not neg
            c = c[1:].strip()
        m = re.match(r"^([A-Za-z_]\w*)\s*==\s*(.+)$", c)
        if m:
            left = env.get(m.group(1))
            right = self.ev(m.group(2), env, where)
            if isinstance(left, Str) and isinstance(right, Str):
                return (left.value == right.value) != neg
            self.fail(where, "`if` comparison that is not constant at the call site",
                      whole)
        if _IDENT.match(c):
            v = env.get(c)
            if isinstance(v, BoolVal):
                return v.value != neg
            self.fail(where, "`if` condition that is not constant at the call site",
                      whole)
        self.fail(where, "unmodelled `if` condition", whole)

    def captured(self, key: str):
        """A capture, by key. Two key spaces, one per capture mechanism.

        `<stem>::<case>` is mechanism A -- the program a single `#[test]` wrote,
        recovered from the directory `run_source` leaves behind. `<fn>(<args>)`
        is mechanism B -- a builder's return value, taken by calling it.
        """
        if self.cap is None:
            return None
        stem, _, rest = key.partition("::")
        if rest and stem == self.src.stem and rest in self.cap.RUNTIME_FORIN:
            return self.cap.RUNTIME_FORIN[rest]
        if key in self.cap.SPREAD:
            return self.cap.SPREAD[key]
        return None

    def call(self, name: str, args: list[str], env: dict, where: str):
        fn = self.src.fns[name]
        if name == "kali_bin":
            self.fail(where, "kali_bin() used as a value", name)

        # A STRING-RETURNING BUILDER THIS EVALUATOR CANNOT RESOLVE is answered
        # by a capture or not at all (rule 9). The condition is derived from the
        # fn's own body -- it calls something outside this file, or builds its
        # value with `format!` -- rather than naming the helpers, so a new
        # builder raises for its capture instead of being silently computed.
        if fn["ret"].replace("'static", "").replace("&", "").strip() in (
                "String", "str") and self.unresolvable(name):
            rendered = ", ".join(self.render_arg(a, env, where) for a in args)
            key = f"{name}({rendered})"
            cap = self.captured(key)
            if cap is None:
                self.fail(where,
                          f"a builder this evaluator cannot resolve and has no "
                          f"capture for -- run t19b5_capture_run.py and register "
                          f"{key!r}", name)
            self.record_reach(name, env["__case__"])
            return Str(cap, "capture")
        child = {"__case__": env["__case__"], "__fn__": name,
                 "__fixtures__": dict(env.get("__fixtures__", {}))}
        if len(args) != len(fn["params"]):
            self.fail(where, f"arity mismatch calling `{name}`", ", ".join(args))
        for p, a in zip(fn["params"], args):
            child[p] = self.ev(a, env, where)
        self.record_reach(name, env["__case__"])
        before = len(self.invocations)
        # A Rust fn's value is its TAIL EXPRESSION -- a final statement with no
        # `;`. It is split off and EVALUATED rather than executed, because a tail
        # is an expression: running it as a statement is how a builder whose body
        # is one bare string literal came back as "statement outside the
        # modelled language".
        stmts = split_statements(fn["masked"])
        tail = None
        # Only a fn with a `->` return type HAS a tail expression. Without this
        # test a trailing `if json_output { … }` in a `()`-returning helper is
        # read as the fn's value and evaluated as an expression, so its whole
        # branch of claims is never run.
        if fn["ret"] and stmts and not stmts[-1].strip().endswith(";") and \
                not re.match(r"^(?:for|while)\b", stmts[-1].strip()):
            tail = stmts.pop()
        for st in stmts:
            self.run_stmt(st, child, f"{where} -> {name}")
        made = self.invocations[before:]
        if tail is not None:
            return self.ev(tail, child, f"{where} -> {name}")
        if made:
            return OutputVal(made[-1])
        return None

    # -- the claim table -------------------------------------------------

    def needle(self, arg: str, env: dict, where: str, whole: str) -> str:
        v = self.ev(arg, env, where)
        if isinstance(v, Str):
            return v.value
        self.fail(where, "needle that does not resolve to a literal", whole)

    def assertion(self, s: str, env: dict, where: str):
        m = re.match(r"^assert(_eq|_ne)?!\s*\(", s)
        kind = m.group(1)
        end = _matching(s, m.end() - 1, "(", ")")
        args = split_top_commas(s[m.end():end - 1])
        if kind == "_ne":
            self.fail(where, "assert_ne! is not in the claim table", s)
        if kind == "_eq":
            self.assert_eq(args, env, where, s)
            return
        self.assert_bool(args[0], env, where, s)

    def assert_eq(self, args, env, where, whole):
        if len(args) < 2:
            self.fail(where, "assert_eq! with fewer than two arguments", whole)
        left = self.ev(args[0], env, where)
        right = self.ev(args[1], env, where)

        # C3 -- exit code
        if isinstance(left, tuple) and left and left[0] == "code":
            if isinstance(right, tuple) and right and right[0] == "Some":
                left[1].add({"kind": "exit", "value": right[1]})
                return
            self.fail(where, "exit code compared to an unmodelled value", whole)

        # C4 -- exact stdout / stderr
        if isinstance(left, StreamVal):
            if isinstance(right, Str):
                left.inv.add({"kind": left.stream, "value": right.value})
                return
            self.fail(where, "stream compared to a non-literal", whole)

        # C9 / C11 -- json
        if isinstance(left, JsonPath):
            if isinstance(right, Str):
                left.inv.add({"kind": "json", "path": left.path,
                              "value": right.value})
                return
            if isinstance(right, BoolVal):
                left.inv.add({"kind": "json", "path": left.path,
                              "value": right.value})
                return
            if isinstance(right, int):
                left.inv.add({"kind": "json", "path": left.path, "value": right})
                return
            if isinstance(right, tuple) and right and right[0] == "json_literal":
                left.inv.add({"kind": "json", "path": left.path,
                              "value": right[1]})
                return
            self.fail(where, "json path compared to an unmodelled value", whole)

        # `source.matches(alias).count()` reaches `member` and raises there.
        self.fail(where, "assert_eq! shape outside the claim table", whole)

    def assert_bool(self, expr: str, env, where, whole):
        e = expr.strip()
        while e.startswith("(") and _matching(e, 0, "(", ")") == len(e):
            e = e[1:-1].strip()

        # C7 -- the rule-11 disjunction, recognised BEFORE the operands are
        # evaluated, because it is one claim rather than two (ruling 14).
        parts = self.split_logical(e, "||")
        if len(parts) > 1:
            if len(parts) != 2:
                self.fail(where, "a disjunction of more than two operands", whole)
            a = self.disjunct(parts[0], env, where, whole)
            b = self.disjunct(parts[1], env, where, whole)
            if a[0] == "exit_failure" and b[0] == "stdout_eq":
                a[1].add({"kind": "or_fail_or_stdout", "stdout": b[1],
                          "source": norm(whole)})
                return
            self.fail(where, "a disjunction outside the modelled shape", whole)

        parts = self.split_logical(e, "&&")
        if len(parts) > 1:
            if len(parts) != 2:
                self.fail(where, "a conjunction of more than two operands", whole)
            for p in parts:
                d = self.disjunct(p, env, where, whole)
                if d[0] == "exit_success":
                    d[1].add({"kind": "exit", "value": "success"})
                elif d[0] == "exit_failure":
                    d[1].add({"kind": "exit", "value": "failure"})
                elif d[0] == "stdout_eq":
                    d[2].add({"kind": "stdout", "value": d[1]})
                else:
                    self.fail(where, "a conjunct outside the modelled shape", whole)
            return

        neg = False
        while e.startswith("!"):
            neg = not neg
            e = e[1:].strip()
        v = self.ev(e, env, where)

        # C1 / C2 -- exit
        if isinstance(v, tuple) and v and v[0] == "success":
            v[1].add({"kind": "exit", "value": "failure" if neg else "success"})
            return
        # C5 / C6 -- stream contains
        if isinstance(v, tuple) and v and v[0] == "contains":
            stream = v[1]
            key = f"{stream.stream}_{'absent' if neg else 'contains'}"
            stream.inv.add({"kind": key, "value": v[2]})
            return
        # C12 -- a `.contains` against a json STRING LEAF is a `json_count`
        # with `at_least = 1` (ruling 3, amended clause 4). Never an exact pin.
        if isinstance(v, tuple) and v and v[0] == "json_contains":
            if neg:
                self.fail(where, "a negated json-leaf `.contains` is not modelled",
                          whole)
            leaf = v[1]
            leaf.inv.add({"kind": "json_count", "path": leaf.path,
                          "needle": v[2], "at_least": 1})
            return
        # C10 -- `json[..].as_array().expect(..).is_empty()`
        if isinstance(v, tuple) and v and v[0] == "is_empty":
            if neg:
                self.fail(where, "a negated `.is_empty()` is not modelled", whole)
            if v[3] != "as_array":
                self.fail(where, "`.is_empty()` on a non-array json node", whole)
            v[1].add({"kind": "json", "path": v[2], "value": []})
            return
        self.fail(where, "assert! shape outside the claim table", whole)

    def disjunct(self, part: str, env, where, whole):
        p = part.strip()
        while p.startswith("(") and _matching(p, 0, "(", ")") == len(p):
            p = p[1:-1].strip()
        m = re.match(r"^(.+?)\s*==\s*(.+)$", p)
        if m and "==" not in m.group(2):
            left = self.ev(m.group(1), env, where)
            right = self.ev(m.group(2), env, where)
            if isinstance(left, StreamVal) and left.stream == "stdout" and \
                    isinstance(right, Str):
                return ("stdout_eq", right.value, left.inv)
            self.fail(where, "an `==` operand outside the modelled shape", whole)
        neg = False
        while p.startswith("!"):
            neg = not neg
            p = p[1:].strip()
        v = self.ev(p, env, where)
        if isinstance(v, tuple) and v and v[0] == "success":
            return ("exit_failure" if neg else "exit_success", v[1])
        self.fail(where, "an operand outside the modelled shape", whole)

    def unresolvable(self, name: str) -> bool:
        """Does this fn build its value out of something this module cannot see?

        True when its body calls an identifier that is neither a fn of this file
        nor one of the handful of std constructors the evaluator models -- i.e. a
        `kali_common::` builder -- or when it builds its value with `format!`
        over such a value. Derived from the body, so a new builder is caught.
        """
        # STRINGS BLANKED. A JS fixture body is full of `console.log(` and
        # `Object.freeze(`; scanning the unblanked text reads every one of them
        # as a call to something outside this file, which makes every literal
        # builder look unresolvable and demands a capture for a fixture that is
        # sitting right there as a literal.
        body = blank_strings(self.src.fns[name]["masked"])
        # RULE 8, at the level of the BUILDER rather than the write. A
        # string-returning fn that assembles its value with `format!` may not be
        # resolved structurally at all: hand-applying Rust's substitution and
        # brace-collapse rules ships a different program, and the real-binary
        # check then verifies the corrupted fixture against itself. Such a fn is
        # answered by a capture or it raises.
        if re.search(r"\bformat!\s*\(", body):
            return True
        known = set(self.src.fns) | {
            "format", "String", "from_utf8_lossy", "Command", "new", "tempdir",
            "expect", "join", "path", "arg", "env", "current_dir", "output",
            "write", "create_dir_all", "match", "if", "for", "while", "let",
            "return", "as_ref", "to_string", "trim_end_matches", "split",
            "matches", "count", "lines", "filter", "collect",
        }
        # ONE PREDICATE, SPELLED AS ONE. This loop used to re-`search` `body` for
        # the very token `finditer` had just produced from `body` -- a condition
        # that is true by construction, so the branch could not fail and read as
        # a check that was doing work. It reduces to exactly this, and saying so
        # is the point: a module whose argument is that its predicates BITE
        # cannot carry a predicate that cannot.
        for m in re.finditer(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(", body):
            tok = m.group(1)
            if tok in known or tok in self.src.consts:
                continue
            return True
        return False

    def render_arg(self, a: str, env: dict, where: str) -> str:
        v = self.ev(a, env, where)
        if isinstance(v, Str):
            return '"' + v.value + '"'
        self.fail(where, "builder argument that is not a literal", a)

    def split_logical(self, e: str, op: str) -> list[str]:
        """Depth-0, string-aware split on `||` / `&&`.

        `&&` and `||` inside a JS fixture (`(true && Array.from)`) are program
        text; splitting on them would invent an operand out of a fixture body.
        """
        spans = [(l["start"], l["end"]) for l in find_string_literals(e)]
        out, cur, depth, i, n, si = [], [], 0, 0, len(e), 0
        while i < n:
            while si < len(spans) and spans[si][1] <= i:
                si += 1
            if si < len(spans) and spans[si][0] <= i < spans[si][1]:
                cur.append(e[i:spans[si][1]])
                i = spans[si][1]
                continue
            c = e[i]
            if c in "([{":
                depth += 1
            elif c in ")]}":
                depth -= 1
            if depth == 0 and e.startswith(op, i):
                out.append("".join(cur))
                cur = []
                i += 2
                continue
            cur.append(c)
            i += 1
        out.append("".join(cur))
        return [p.strip() for p in out if p.strip()]


# ---------------------------------------------------------------------------
# The cross-crate environment-variable name, read rather than assumed
# ---------------------------------------------------------------------------

_ENV_NAME: list = []


def browser_harness_env_name() -> str:
    """`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV`, read from its crate.

    Two of these sources spell the variable out and three reach it through this
    constant. Reading the crate is what keeps those two spellings from drifting
    apart in a migrated case file: a rename in `kali_runtime_contract` fails
    here loudly instead of shipping the old name.
    """
    if _ENV_NAME:
        return _ENV_NAME[0]
    root = os.path.join(REPO, "crates/kali_runtime_contract/src")
    pat = re.compile(
        r"BROWSER_HARNESS_COMMAND_ENV\s*:\s*&(?:'static\s+)?str\s*=\s*\"([^\"]+)\"")
    for dirpath, _dirs, files in os.walk(root):
        for f in files:
            if not f.endswith(".rs"):
                continue
            m = pat.search(open(os.path.join(dirpath, f), encoding="utf-8").read())
            if m:
                _ENV_NAME.append(m.group(1))
                return m.group(1)
    raise UnknownShape(
        "kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV not found in its crate")


# ---------------------------------------------------------------------------
# The residue scan -- closure over CLAIMS, not over macros
# ---------------------------------------------------------------------------

_PERMITTED_STMT = [
    r"^let\b",
    # A bare `Command::new(..)…output().expect(..)` -- the TAIL EXPRESSION of a
    # runner helper (`runtime_forin::run_source`), which is the same statement
    # every other file writes as `let output = …;`. Spawning is not a claim; the
    # claims are the assertions its caller makes on the value.
    r"^Command::new\s*\(",
    r"^static\s+[A-Z]",
    r"^(?:std::)?fs::write\s*\(",
    r"^(?:std::)?fs::create_dir_all\s*\(",
    r"^for\b", r"^if\b", r"^else\b", r"^while\b",
    r"^assert(?:_eq|_ne)?!\s*\(",
    r"^return\b",
    r"^match\b",
]

# A permitted CALL is blanked only to the closing paren of that call, so
# anything chained onto it stays in the residue -- batch 3 had to split the
# forms this way after a `.contains` chained onto a permitted call went unseen.
_PERMITTED_CALL = [
    r"\bCommand::new\s*\(",
    r"\btempdir\s*\(",
    r"\bkali_bin\s*\(",
    r"\bformat!\s*\(",
    r"\bString::from_utf8_lossy\s*\(",
    r"\bserde_json::from_slice\s*\(",
    r"\bstd::env::temp_dir\s*\(",
    r"\bstd::process::id\s*\(",
]

_CLAIM_TOKENS = [
    (r"\bpanic!\s*\(", "a claim written as `panic!`"),
    (r"\bdebug_assert\w*!\s*\(", "a `debug_assert`"),
    (r"\bmatches!\s*\(", "a `matches!` claim"),
    (r"\.expect\s*\(", "a claim carried by `.expect()`"),
    (r"\.unwrap\s*\(", "a claim carried by `.unwrap()`"),
    (r"\.contains\s*\(", "a bare `.contains` outside any assertion"),
    (r"\.matches\s*\(", "a bare `.matches` outside any assertion"),
    (r"\.status\b", "a use of `.status` outside any assertion"),
    (r"\.stdout\b", "a use of `.stdout` outside any assertion"),
    (r"\.stderr\b", "a use of `.stderr` outside any assertion"),
]


def _blank_span(s: str, lo: int, hi: int) -> str:
    return s[:lo] + "".join(c if c == "\n" else " " for c in s[lo:hi]) + s[hi:]


def residual_claims(fn_masked: str, line_base: int) -> list[str]:
    text = fn_masked
    for m in reversed(list(re.finditer(r"\bassert(?:_eq|_ne)?!\s*\(", text))):
        try:
            end = _matching(text, m.end() - 1, "(", ")")
        except UnknownShape:
            continue
        text = _blank_span(text, m.start(), end)

    def blank_stmts(s: str, lo: int, hi: int) -> str:
        base = lo
        for st in split_statements(s[lo:hi]):
            idx = s.find(st, base, hi)
            if idx < 0:
                continue
            base = idx + len(st)
            stripped = st.strip()
            if any(re.match(rx, stripped) for rx in _PERMITTED_STMT):
                if re.match(r"^(?:for|if|while|else|match)\b", stripped):
                    j = idx
                    while True:
                        k = s.find("{", j, idx + len(st))
                        if k < 0:
                            break
                        try:
                            end = _matching(s, k, "{", "}")
                        except UnknownShape:
                            break
                        s = blank_stmts(s, k + 1, end - 1)
                        j = end
                    # the head of the `for`/`if` itself is not a claim site
                    head_end = s.find("{", idx)
                    if 0 <= head_end < idx + len(st):
                        s = _blank_span(s, idx, head_end)
                    continue
                s = _blank_span(s, idx, idx + len(st))
                continue
        return s

    out_text = blank_stmts(text, 0, len(text))
    for pat in _PERMITTED_CALL:
        while True:
            m = re.search(pat, out_text)
            if not m:
                break
            i = out_text.index("(", m.end() - 1)
            try:
                end = _matching(out_text, i, "(", ")")
            except UnknownShape:
                break
            # An `.expect(..)` / `.unwrap()` chained DIRECTLY onto a permitted
            # constructor is that constructor's unwrap, not a claim about the
            # program under test: `serde_json::from_slice(&out.stdout)
            # .expect("json stdout")` says stdout must parse as JSON, which every
            # `json.*` claim in the rendered case re-asserts by construction.
            # Anything else chained on stays in the residue -- which is the split
            # batch 3 had to introduce and is why the blanking stops here and not
            # at the end of the chain.
            e2 = re.match(r"\s*\.\s*(?:expect|unwrap)\s*\(", out_text[end:])
            if e2:
                end = _matching(out_text, end + e2.end() - 1, "(", ")")
            out_text = _blank_span(out_text, m.start(), end)
    out = []
    for rx, why in _CLAIM_TOKENS:
        for m in re.finditer(rx, out_text):
            line = line_base + out_text.count("\n", 0, m.start())
            out.append(f"{why} @~{line}: "
                       f"{norm(out_text[max(0, m.start() - 40):m.start() + 40])!r}")
    return out


# ---------------------------------------------------------------------------
# Driver
# ---------------------------------------------------------------------------

# A stem whose `.rs` is a U4 TRIM reads its PRE-TRIM blob, not the working tree.
# The case file was migrated from the whole file, and its `SOURCE REF:` names
# exactly this commit; reading the trimmed file instead would make the generator
# emit an empty case file and call itself a fixed point.
PRE_TRIM = {
    "for_of_array_iteration_spread": "47e9b083c61e32c972727189a580d1e9cacb856c",
}


class PreTrimSource(Source):
    """`Source`, over the bytes at a pinned ref rather than the working tree."""

    def __init__(self, stem: str, ref: str):
        self.stem = stem
        self.path = os.path.join(TESTS, stem + ".rs")
        self.text = subprocess.run(
            ["git", "show", f"{ref}:crates/kali_cli/tests/{stem}.rs"],
            cwd=REPO, capture_output=True, check=True).stdout.decode("utf-8")
        self.masked = blank_line_comments(self.text)
        self.skeleton = blank_strings(self.masked)
        self.fns = {}
        self.consts = {}
        self.tests = []
        self._index()


def extract(stem: str, captures=None) -> dict:
    if captures is None:
        import t19b5_captures as captures      # the default, not an option
    src = PreTrimSource(stem, PRE_TRIM[stem]) if stem in PRE_TRIM else Source(stem)
    ev = Evaluator(src, captures)
    retained = RETAINED.get(stem, set())
    per_test = []
    for t in src.tests:
        if t["name"] in retained:
            continue
        env = {"__case__": t["name"], "__fn__": t["name"], "__fixtures__": {}}
        before = len(ev.invocations)
        ev.run_block(t["masked"], env, f"{stem}.rs::{t['name']}")
        made = ev.invocations[before:]
        if not made:
            raise UnknownShape(f"{stem}.rs::{t['name']}: runs no kali command")
        for inv in made:
            if not inv.claims:
                raise UnknownShape(
                    f"{stem}.rs::{t['name']}: a command with no claim at all")
            if not inv.fixtures:
                raise UnknownShape(
                    f"{stem}.rs::{t['name']}: a command with no fixture")
        res = residual_claims(t["masked"], t["attr_line"])
        if res:
            raise UnknownShape(
                f"{stem}.rs::{t['name']}: claim(s) outside the table:\n    "
                + "\n    ".join(res))
        per_test.append({"name": t["name"], "invocations": made})
    for name, fn in src.fns.items():
        if name == "kali_bin":
            continue
        if name in _fns_only_reached_by(src, retained):
            continue
        res = residual_claims(fn["masked"], fn["line"])
        if res:
            raise UnknownShape(
                f"{stem}.rs::{name} (helper): claim(s) outside the table:\n    "
                + "\n    ".join(res))
    return {"stem": stem, "src": src, "ev": ev, "tests": per_test,
            "retained": sorted(retained)}


def _fns_only_reached_by(src: Source, retained: set) -> set:
    """Helpers reachable ONLY from a retained `#[test]`.

    A U4 trim moves those out of the migrated half entirely, so scanning them
    for residual claims would report the retained test's own un-migratable
    construct as this batch's failure. The set is computed from the call graph
    rather than named, so it cannot go stale against a source edit.
    """
    if not retained:
        return set()

    def calls(body: str) -> set:
        return {n for n in src.fns if re.search(r"\b" + re.escape(n) + r"\s*\(", body)}

    def closure(seed: set) -> set:
        seen, stack = set(), list(seed)
        while stack:
            cur = stack.pop()
            if cur in seen or cur not in src.fns:
                continue
            seen.add(cur)
            stack += list(calls(src.fns[cur]["masked"]))
        return seen

    kept, migrated = set(), set()
    for t in src.tests:
        target = kept if t["name"] in retained else migrated
        target |= calls(t["masked"])
    return closure(kept) - closure(migrated)


def select() -> list[str]:
    """The work list, re-derived rather than tabulated (ruling 13).

    A CLEAN, unmigrated target qualifies iff it is NOT one of the four `DECLINED`
    dead-`json_output`-branch targets -- and that exclusion is re-MEASURED here,
    not read off the set: `check_declined()` proves the property still holds, so
    a source that grew a reachable `json_output = true` call site would fail this
    predicate instead of staying silently declined.
    """
    import glob
    import subprocess
    pat = re.compile(r"Migrated from tests/([A-Za-z0-9_/]+)\.rs")
    migrated = set()
    for p in glob.glob(os.path.join(TESTS, "cases/*/*.toml")):
        migrated |= set(pat.findall(open(p, encoding="utf-8").read()))
    # This batch's own targets are excluded from the exclusion, so the predicate
    # answers the same list before and after its own commit. Batch 4 shipped the
    # other version first: it returns the right list ONCE and `[]` forever after.
    migrated -= set(STEMS)
    clean = subprocess.run(
        [sys.executable, os.path.join(REPO, "tools/migration/screen_candidates.py"),
         "--list-clean"], capture_output=True, text=True, check=True).stdout.split()
    # A stem this batch TRIMS is added back for the same reason its own case
    # files are excluded from `migrated`: the trim is this batch's own act, and
    # after it lands the screen correctly BLOCKS the stem as `S27_self_documented`
    # -- U3's marker doing its job. Without this the predicate answers the right
    # list once and a different one forever after, which is the defect batch 4
    # shipped in the mirror-image direction.
    out = []
    for s in sorted(set(clean) | set(PRE_TRIM)):
        if s in migrated or s in DECLINED:
            continue
        out.append(s)
    return out


def dead_bool_branches(text: str) -> list[dict]:
    """Every `bool` parameter that is dead AND guards claims, with its line.

    BOTH HALVES ARE MEASURED, and the second half is the one that was missing.
    A parameter qualifies iff:

      1. every call site passes it the same literal `false` -- so the `true`
         branch is UNREACHABLE; AND
      2. at least one `if <param> { … }` block inside that fn CARRIES CLAIMS
         (an `assert*!`) -- so something is actually dead in there.

    Half 1 alone is not the ground controller ruling R1 settles. An
    always-`false` bool guarding an EMPTY block blocks nothing: the audit has no
    dead literal to demand and the target would be migratable. Measuring only
    half 1 is what this function did for a round, while its docstring claimed
    both -- so the gate four retention headers cite as their re-derivation was
    weaker than the sentence they cite.

    Returns `[{"helper", "param", "line", "blocks", "claims"}]`, `line` being the
    line of the first claim-carrying `if <param> {` -- ruling 9's "BY NAME AND
    LINE" is answered from here rather than promised in prose.
    """
    # TWO VIEWS AT THE SAME OFFSETS. Structure is found in the fully blanked
    # text (a `{` inside a JS fixture is not a block); argument CONTENT is
    # read from the comment-blanked text, where literals are still there.
    # Reading arguments off the blanked view drops every string argument to
    # the empty string, and a positional split then shifts the bool's index
    # -- which silently reported one dead branch per file instead of two.
    masked = blank_strings(blank_line_comments(text))
    live = blank_line_comments(text)
    dead = []
    for m in re.finditer(r"(?:^|\n)fn\s+([a-z_][a-z0-9_]*)\s*\(", masked):
        name = m.group(1)
        sig_end = _matching(masked, m.end() - 1, "(", ")")
        sig = masked[m.end():sig_end - 1]
        order = [p.split(":")[0].strip() for p in split_top_commas(sig)]
        bools = [p.split(":")[0].strip()
                 for p in split_top_commas(sig) if ": bool" in p]
        body_open = masked.find("{", sig_end)
        body_end = (_matching(masked, body_open, "{", "}")
                    if body_open >= 0 else len(masked))
        for b in bools:
            idx = order.index(b)
            seen = set()
            for c in re.finditer(r"\b" + re.escape(name) + r"\s*\(", masked):
                if m.start() <= c.start() < sig_end:
                    continue
                cend = _matching(masked, c.end() - 1, "(", ")")
                args = split_top_commas(live[c.end():cend - 1])
                if idx < len(args):
                    seen.add(args[idx].strip())
            if not (seen and seen <= {"false"}):
                continue
            # HALF 2: the guarded block must carry a claim.
            blocks, claims, first = 0, 0, None
            for g in re.finditer(r"\bif\s+" + re.escape(b) + r"\s*\{",
                                 masked[body_open:body_end]):
                blocks += 1
                start = body_open + g.end() - 1
                end = _matching(masked, start, "{", "}")
                n = len(re.findall(r"\bassert(?:_eq|_ne)?!\s*\(",
                                   masked[start:end]))
                claims += n
                if n and first is None:
                    first = body_open + g.start()
            if not claims:
                continue
            dead.append({"helper": name, "param": b,
                         "line": masked[:first].count("\n") + 1,
                         "blocks": blocks, "claims": claims})
    return dead


def check_declined() -> dict:
    """Re-measure the ground for declining the four, rather than citing it.

    For each: every `bool` parameter of every helper, against the literal passed
    at EVERY call site, AND against the contents of the block that parameter
    guards. The target is correctly declined iff some parameter is passed only
    `false` while its `if <param> {…}` block carries claims -- dead literals,
    controller ruling R1's shape. Both halves are in `dead_bool_branches`, which
    carries its own known positives (`--selftest`).
    """
    out = {}
    for stem in sorted(DECLINED):
        text = open(os.path.join(TESTS, stem + ".rs"), encoding="utf-8").read()
        dead = dead_bool_branches(text)
        if not dead:
            raise UnknownShape(
                f"{stem}.rs: DECLINED, but no dead bool branch found -- the "
                f"ground for declining it no longer holds and the disposition "
                f"must be re-adjudicated, not inherited")
        out[stem] = dead
    return out


def format_dead(d: dict) -> str:
    return f"{d['helper']}({d['param']}) line {d['line']}"


_SELFTEST_LIVE = """
fn helper(source: &str, json_output: bool) {
    if json_output {
        assert_eq!(json["schemaVersion"], 1);
    }
    assert!(out.status.success());
}

#[test]
fn t() {
    helper("x", false);
}
"""

_SELFTEST_EMPTY_BLOCK = _SELFTEST_LIVE.replace(
    'assert_eq!(json["schemaVersion"], 1);', 'let _unused = source.len();')

_SELFTEST_REACHABLE = _SELFTEST_LIVE.replace('helper("x", false);',
                                             'helper("x", false);\n    helper("y", true);')


def _selftests() -> int:
    """Known positives for BOTH halves of `dead_bool_branches`.

    A gate four retention headers cite as their re-derivation has to be
    falsifiable, and until this round its second half was a sentence rather than
    a measurement. Each arm below is an input that must make the predicate say
    NO; the first is the input that must make it say YES, so a predicate stuck
    on either answer fails here.
    """
    ok = 0
    live = dead_bool_branches(_SELFTEST_LIVE)
    if [ (d["helper"], d["param"]) for d in live ] != [("helper", "json_output")]:
        raise UnknownShape(f"the control shape is not measured dead: {live}")
    if live[0]["line"] != 3:
        raise UnknownShape(f"the guarded block's line is wrong: {live}")
    ok += 1
    print("  ok  selftest: an always-false bool guarding a claim IS reported, "
          "with the guard's line")
    if dead_bool_branches(_SELFTEST_EMPTY_BLOCK):
        raise UnknownShape(
            "an always-false bool guarding a block with NO claim is reported "
            "dead -- half 2 of the predicate is not measuring anything")
    ok += 1
    print("  ok  selftest: an always-false bool guarding NO claim is not "
          "reported (half 2 fires)")
    if dead_bool_branches(_SELFTEST_REACHABLE):
        raise UnknownShape(
            "a bool reached with `true` is reported dead -- half 1 of the "
            "predicate is not measuring anything")
    ok += 1
    print("  ok  selftest: a bool reached with `true` is not reported "
          "(half 1 fires)")
    if ok != 3:
        raise UnknownShape("a selftest did not run")
    return ok


def census() -> int:
    rows, tot_fn, tot_inv, tot_claims = [], 0, 0, 0
    got = select()
    if got != sorted(STEMS):
        print("WORK LIST MOVED", file=sys.stderr)
        print(f"  derived: {got}", file=sys.stderr)
        print(f"  STEMS:   {sorted(STEMS)}", file=sys.stderr)
        return 1
    declined = check_declined()
    for stem in sorted(STEMS):
        d = extract(stem)
        kinds = {}
        for t in d["tests"]:
            for inv in t["invocations"]:
                for c in inv.claims:
                    kinds[c["kind"]] = kinds.get(c["kind"], 0) + 1
        ninv = sum(len(t["invocations"]) for t in d["tests"])
        nclaim = sum(len(i.claims) for t in d["tests"] for i in t["invocations"])
        rows.append((stem, len(d["tests"]), ninv, nclaim, kinds,
                     len(d["retained"])))
        tot_fn += len(d["tests"])
        tot_inv += ninv
        tot_claims += nclaim
    for stem, nfn, ninv, nclaim, kinds, nret in rows:
        shapes = " ".join(f"{k}={v}" for k, v in sorted(kinds.items()))
        extra = f"  (+{nret} retained)" if nret else ""
        print(f"  {stem:45s} {nfn:3d} fn(s) {ninv:4d} invocation(s) "
              f"{nclaim:4d} claim(s){extra}")
        print(f"  {'':45s} {shapes}")
    print()
    print(f"{len(rows)} target(s), {tot_fn} migrated `#[test]` fn(s), "
          f"{tot_inv} invocation(s), {tot_claims} claim(s)")
    print(f"{len(declined)} target(s) DECLINED, each re-measured:")
    for stem, dead in sorted(declined.items()):
        print(f"  {stem:52s} dead: {', '.join(format_dead(d) for d in dead)}")
    print("EXTRACTOR CENSUS OK")
    return 0


def main(argv: list[str]) -> int:
    if argv and argv[0] == "--list":
        for s in select():
            print(s)
        return 0
    if argv and argv[0] == "--declined":
        for stem, dead in sorted(check_declined().items()):
            print(f"{stem}\t{','.join(format_dead(d) for d in dead)}")
        return 0
    if argv and argv[0] == "--selftest":
        _selftests()
        print("EXTRACTOR SELFTEST OK")
        return 0
    return census()


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
