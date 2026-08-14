#!/usr/bin/env python3
r"""Resolve what a batch-8B source helper asserts, for one concrete invocation.

THE PROBLEM THIS SOLVES. Nine of batch 8B's thirteen targets funnel every
`#[test]` fn through a helper whose assertion set depends on its own parameters:

    for json_output in [false, true] { ...
        if json_output { <envelope claims> } else { <text claims> }
        if command == "run" { <exitCode claims> } else { <total/passed claims> } }

A generator that hand-lists "the run/json case asserts these seven paths" is
transcribing a branch it read by eye, over 375 invocations, with no gate that
can see a mistake: `audit-case-migration.py` only checks that literals are not
DROPPED, so a claim attached to the wrong branch is invisible to it, and the
trial still passes. So the branches are resolved here instead -- the caller
supplies the binding the call site supplies (`command`, `json_output`,
`with_browser_api_surface_manifest`, ...), and this module returns the claims
that survive it.

WHAT IS DERIVED VS DECIDED. This module derives; it decides nothing. Whether a
claim becomes `stdout_contains` or an exact pin is controller ruling 3's
question and lives in the generator. A condition this module cannot evaluate is
a RAISE, never a guess -- an unevaluated `if` would silently contribute both
branches' claims, which is the same "asserts everything, discriminates nothing"
failure the U2 hazard produces.

INLINED HELPERS. `assert_empty_thread_topology(&json["payload"]["threadTopology"])`
and the two `assert_browser_runtime_rejection_*` helpers assert through a
parameter, so their claims are read from their own bodies and re-prefixed with
the path the call site passes. The call site's literal arguments bind the
callee's parameters (`expected_origin`), so a source that changes `"cli"` to
something else changes the emitted pin rather than being ignored.
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

from batch8b_extract import claims_in, fn_body, flat_ws, or_groups  # noqa: E402


def _eval_cond(cond, bind):
    cond = cond.strip()
    if cond in bind:
        return bool(bind[cond])
    m = re.fullmatch(r'([a-z_]+)\s*==\s*"([^"]*)"', cond)
    if m and m.group(1) in bind:
        return bind[m.group(1)] == m.group(2)
    raise AssertionError(
        f"cannot evaluate `if {cond}` -- add the binding rather than letting both "
        "branches contribute claims")


def resolve(body, bind):
    """`body` with every top-level-or-nested `if` resolved under `bind`."""
    out = []
    i = 0
    n = len(body)
    while i < n:
        m = re.compile(r"\bif\s+([^\n{]+?)\s*\{").search(body, i)
        if not m:
            out.append(body[i:])
            break
        out.append(body[i:m.start()])
        open_at = m.end() - 1
        close = _match(body, open_at)
        taken = body[open_at + 1:close]
        rest = body[close + 1:]
        em = re.match(r"\s*else\s*\{", rest)
        other = ""
        after = close + 1
        if em:
            o2 = close + 1 + em.end() - 1
            c2 = _match(body, o2)
            other = body[o2 + 1:c2]
            after = c2 + 1
        chosen = taken if _eval_cond(m.group(1), bind) else other
        out.append(resolve(chosen, bind))
        i = after
    return "".join(out)


def _match(text, open_at):
    depth, i = 0, open_at
    while i < len(text):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    raise AssertionError("unbalanced brace")


INLINE_PREFIX = {
    "assert_empty_thread_topology": "payload.threadTopology",
}


def claims_for(rs_text, helper, bind, *, extra_bind=None):
    """Every claim `helper` makes under `bind`, with inlined helpers expanded."""
    return claims_of(rs_text, fn_body(rs_text, helper)[0], bind, extra_bind=extra_bind)


def claims_of(rs_text, body, bind, *, extra_bind=None):
    """`claims_for`, given a body rather than a helper name.

    Three of this batch's sources build their `Command` inline in each `#[test]`
    fn instead of behind a shared helper, so the same resolution has to run
    against a test body. One implementation, two entry points -- two would be
    two chances to disagree about what a branch asserts.
    """
    body = resolve(body, bind)
    out = claims_in(body)
    # `assert_empty_thread_topology(&json["payload"]["threadTopology"])`
    for m in re.finditer(r'assert_empty_thread_topology\(&(?:json|value)((?:\["[^"]*"\])+)\)', body):
        parts = re.findall(r'\["([^"]*)"\]', m.group(1))
        sub, _ = fn_body(rs_text, "assert_empty_thread_topology")
        for c in claims_in(sub, prefix=".".join(parts)):
            if c[0] == "json":
                out.append(("json", c[1].replace("value.", ""), c[2]))
    # `assert_browser_runtime_rejection_text(&stderr)`
    for m in re.finditer(r"assert_browser_runtime_rejection_text\(&stderr\)", body):
        sub, _ = fn_body(rs_text, "assert_browser_runtime_rejection_text")
        out.extend(claims_in(sub))
    # `assert_browser_runtime_rejection_json(&json, "<origin>")`
    for m in re.finditer(r'assert_browser_runtime_rejection_json\(&json,\s*(?:"([^"]*)"|(\w+))\)', body):
        origin = m.group(1)
        if origin is None:
            origin = (extra_bind or {}).get(m.group(2))
            if origin is None:
                raise AssertionError(
                    f"assert_browser_runtime_rejection_json is passed `{m.group(2)}`; bind it")
        sub, _ = fn_body(rs_text, "assert_browser_runtime_rejection_json")
        for c in claims_in(sub):
            if c[0] == "json" and c[2] == "<expected_origin>":
                out.append(("json", c[1], origin))
            else:
                out.append(c)
    # `assert_browser_wasm_threads_rejection(&stderr)`
    for m in re.finditer(r"assert_browser_wasm_threads_rejection\(&stderr\)", body):
        sub, _ = fn_body(rs_text, "assert_browser_wasm_threads_rejection")
        out.extend(claims_in(sub))
        for group in or_groups(sub):
            out.append(("or_group", tuple(group)))
    # `<command>` placeholders resolve from the binding
    resolved = []
    for c in out:
        if c[0] == "json" and c[2] == "<command>":
            resolved.append(("json", c[1], bind["command"]))
        else:
            resolved.append(c)
    # dedupe, order-preserving
    # Keyed on the repr because a parsed `serde_json::json!` value is a list or
    # dict and therefore unhashable; the repr is injective enough for dedupe and
    # keeps the claim tuples themselves intact.
    seen, uniq = set(), []
    for c in resolved:
        key = repr(c)
        if key not in seen:
            seen.add(key)
            uniq.append(c)
    return uniq


def or_needles(claims):
    return [c[1] for c in claims if c[0] == "or_group"]
