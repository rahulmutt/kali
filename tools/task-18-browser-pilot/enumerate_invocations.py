#!/usr/bin/env python3
"""Enumerate every real helper invocation in a browser_*.rs target.

Rule 7 requires the matrix arithmetic to close exactly: `total invocations ==
cases x axis product`. Doing that by eye is how a batch invents an untested
combination, so this derives it mechanically -- it parses each `#[test]` fn,
resolves the `assert_*` helper call it makes (including calls inside `for`
loops over literal tuple/array lists), and prints the resulting argument
tuples grouped by helper.

Deliberately narrow: it understands the shapes this batch's files actually
use (a direct helper call with literal/parameterless-fn arguments, and a `for`
loop over a literal array of tuples or of strings, optionally nested inside a
`for x in [false, true]`). Anything it cannot parse is reported as UNPARSED
rather than silently skipped -- a silent skip would under-count invocations and
make a broken matrix look closed.

Usage: python3 enumerate_invocations.py FILE.rs [FILE.rs ...]
"""

import re
import sys


def strip_block_comments_and_strings(text):
    """Mask string/raw-string/comment content so brace and `for` scanning
    cannot be fooled by a `for` or `{` inside a JS fixture."""
    out = []
    i, n = 0, len(text)
    while i < n:
        if text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j == -1 else j
            out.append(" " * (j - i))
            i = j
            continue
        if text.startswith("/*", i):
            j = text.find("*/", i + 2)
            j = n if j == -1 else j + 2
            out.append(" " * (j - i))
            i = j
            continue
        m = re.match(r'r(#*)"', text[i:])
        if m:
            hashes = m.group(1)
            close = '"' + hashes
            j = text.find(close, i + len(m.group(0)))
            j = n if j == -1 else j + len(close)
            out.append(" " * (j - i))
            i = j
            continue
        if text[i] == '"':
            j = i + 1
            while j < n:
                if text[j] == "\\":
                    j += 2
                    continue
                if text[j] == '"':
                    j += 1
                    break
                j += 1
            out.append(" " * (j - i))
            i = j
            continue
        out.append(text[i])
        i += 1
    return "".join(out)


def test_fn_bodies(text, masked):
    """[(fn_name, body_text_from_original, start_line)] for every #[test] fn."""
    results = []
    for m in re.finditer(r"#\[test\]", masked):
        sig = re.compile(r"fn\s+([a-zA-Z0-9_]+)\s*\(").search(masked, m.end())
        if not sig:
            continue
        brace = masked.find("{", sig.end() - 1)
        depth, i, n = 0, brace, len(masked)
        while i < n:
            if masked[i] == "{":
                depth += 1
            elif masked[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        results.append((sig.group(1), text[brace + 1:i], text[:brace].count("\n") + 1))
    return results


def split_args(s):
    """Top-level comma split, brace/paren/bracket aware."""
    parts, depth, cur = [], 0, ""
    for ch in s:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        parts.append(cur.strip())
    return parts


CALL = re.compile(r"\b(assert_[a-z0-9_]+)\s*\(")
TUPLE_ROW = re.compile(r"\(([^()]*)\)")


def invocations(body):
    """[(helper, [arg, ...])], expanding literal `for` loops."""
    out, unparsed = [], []

    loops = re.findall(
        r"for\s+(\([^)]*\)|[a-zA-Z0-9_]+)\s+in\s+\[(.*?)\]\s*\{",
        body, re.S)

    # EVERY assert_* call in the body, not just the first. Taking only the
    # first silently undercounts a body like
    #     for filename in ["app.jsx", "app.tsx"] {
    #         helper(filename, false);
    #         helper(filename, true);
    #     }
    # as 2 invocations instead of 4, with nothing reported as UNPARSED -- so a
    # matrix declared off that number looks closed and is not. Found by the
    # batch 4 group C implementer on browser_math_expm1_log1p_bracketed_root.rs
    # and browser_math_expm1_log1p_mixed_root.rs, after this tool's numbers had
    # already been quoted into three dispatch briefs.
    calls = []
    for call in CALL.finditer(body):
        helper = call.group(1)
        depth, i = 0, call.end() - 1
        while i < len(body):
            if body[i] == "(":
                depth += 1
            elif body[i] == ")":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        calls.append((helper, split_args(body[call.end():i])))

    if not calls:
        return out, unparsed

    if not loops:
        out.extend(calls)
        return out, unparsed

    # Resolve loop variables. Only literal lists are handled.
    bindings = []
    for var, listing in loops:
        names = [v.strip() for v in var.strip("()").split(",")] if var.startswith("(") else [var]
        rows = []
        if var.startswith("("):
            for row in TUPLE_ROW.findall(listing):
                rows.append([c.strip() for c in split_args(row)])
        else:
            rows = [[v.strip()] for v in split_args(listing)]
        if not rows:
            unparsed.append(f"unparsed loop list for `{var}`")
        bindings.append((names, rows))

    combos = [[]]
    for names, rows in bindings:
        combos = [c + [(names, r)] for c in combos for r in rows]

    for combo in combos:
        env = {}
        for names, row in combo:
            for nm, val in zip(names, row):
                env[nm] = val
        for helper, args in calls:
            out.append((helper, [env.get(a, a) for a in args]))
    return out, unparsed


def main(paths):
    for path in paths:
        text = open(path).read()
        masked = strip_block_comments_and_strings(text)
        fns = test_fn_bodies(text, masked)
        print(f"##### {path}")
        print(f"  #[test] fns: {len(fns)}")
        groups, total, problems = {}, 0, []
        for name, body, line in fns:
            invs, unparsed = invocations(body)
            if not invs:
                problems.append(f"UNPARSED (no helper call found): {name} :{line}")
            for helper, args in invs:
                groups.setdefault(helper, []).append(tuple(args))
                total += 1
            problems.extend(f"{name} :{line}: {u}" for u in unparsed)
        for helper, rows in sorted(groups.items()):
            print(f"  {helper}: {len(rows)} invocation(s)")
            for r in sorted(set(rows)):
                dupes = rows.count(r)
                mark = f"  x{dupes}" if dupes > 1 else ""
                print(f"      {r}{mark}")
        print(f"  TOTAL INVOCATIONS: {total}")
        for p in problems:
            print(f"  !! {p}")
        print()


if __name__ == "__main__":
    main(sys.argv[1:])
