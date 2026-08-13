r"""Mechanical comment-coverage checker for the Task 18 browser/ pilot.

Fixed after pilot round-1 review (finding 4): the original version pooled
the file's `#` header and every case's `rationale` into ONE blob and checked
substring membership in that union -- so a comment paragraph mentioned once,
anywhere (even only in the header), was reported as "covered" even if not
one of N cases actually carried it in its own `rationale`. That is a
materially weaker guarantee than "every case's rationale carries the prose
that produced it": a reader looking at a single failing trial only ever sees
that ONE case's `rationale`, not the file header, so a header-only mention
is invisible to them.

This version checks each non-divider comment paragraph independently
against EVERY case's own `rationale` (not the header, and not the pooled
union) -- i.e. genuine per-case coverage, matching the house rule "helper
`///` docs reach every case that helper produced" applied to every
substantive comment block, not just doc comments. This is a deliberate
simplification for THIS pilot's corpus: none of the six files has more than
one distinct helper-attributable comment block, and where one exists, it
applies to every case in that file (confirmed by direct reading, file by
file). A file where two different helpers each produce a disjoint subset of
cases, with two independent comment blocks, would need real per-helper
attribution (which case came from which comment) that this version does not
attempt -- flagged here rather than silently assumed away; batches 2-8
should extend this if that shape shows up.

The file's `#` header is still parsed but is no longer treated as
coverage on its own -- it can duplicate case rationale content, but it can't
substitute for it.
"""
import os
import re
import sys
import tomllib
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from submodules import submodule_paths  # noqa: E402


GUARD_NOTE_TEXT = """RAW-STRING OPENER, PREFIX- AND BOUNDARY-CORRECT. This is one instance of a
    class enumerated repo-wide by Task 19 batch 4 and gated by
    `inst2_probes.probe_raw_string_recogniser_class`, which fails if a site is
    added without being declared. Two failure directions, both live before that
    enumeration: UNDER-recognition, when the recogniser keys on `r` and so does
    not admit the `b`/`c` of a byte or C raw string (its interior is then read
    as live code); and OVER-recognition, when it has no left word boundary and
    so opens a raw string on the trailing `r` of an ordinary word before a quote
    (`"operator"`). `b"`/`c"` without the `r` are ESCAPED literals and must keep
    falling through to the plain-string path."""


def _raw_string_spans(text):
    """Line numbers (1-based) that lie inside a raw-string literal.

    A JS/TS fixture body is written as a raw string precisely because its
    interior can hold anything, `//` included -- a `// kali-tree-shake:` marker,
    a JS comment, a URL. Those are program text, not source prose, and rule 12
    has nothing to say about them.

    %s

    OVER-recognition was the live defect HERE, and it is a FALSE GREEN in the
    rule-12 gate: with no left boundary and no plain-string branch, the `r` of
    `assert!(s.contains("operator"))` opened a raw string that ran to the next
    quote, so every trailing comment inside that span vanished from the checked
    population. Measured before the fix, on
    `assert!(s.contains("operator"));` followed by `let x = 1; // a genuine
    trailing comment`: `extract_trailing_comments` returned []; with the needle
    spelled `"OPERATOR"` it returned the comment. That is U16's blind spot
    re-opened by a different mechanism.
    """ % GUARD_NOTE_TEXT
    inside = set()
    i, n, line = 0, len(text), 1
    while i < n:
        c = text[i]
        if c == "\n":
            line += 1
            i += 1
            continue
        prefix = 1 if (c in "bc" and i + 1 < n and text[i + 1] == "r") else 0
        head = i + prefix
        m = re.match(r'r(#*)"', text[head:]) if head < n else None
        if m and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")):
            close = '"' + m.group(1)
            end = text.find(close, head + m.end())
            end = n if end == -1 else end + len(close)
            for _ in range(text.count("\n", i, end)):
                inside.add(line)
                line += 1
            inside.add(line)
            i = end
            continue
        if c == '"':
            # A PLAIN string, skipped as a unit. Without this branch the scan
            # walks INTO the literal and meets any `r"` its interior spells --
            # which is how `"operator"` came to open a raw string.
            j = i + 1
            while j < n:
                if text[j] == "\\":
                    j += 2
                    continue
                if text[j] == '"':
                    j += 1
                    break
                if text[j] == "\n":
                    break
                j += 1
            line += text.count("\n", i, j)
            i = j
            continue
        i += 1
    return inside


def extract_trailing_comments(text):
    """`(line, text)` for every `// ...` that FOLLOWS code on its line.

    THE RULE-12 GATE'S BLIND SPOT, CLOSED. `extract_comment_paragraphs` matches
    a `^\\s*//` anchor, so a comment sharing a line with code was invisible to it -- not
    reported missing, not reported at all. That is a false green, and the
    dangerous direction: a source comment could be dropped in migration and this
    checker would say nothing. Found by review on
    `heap_grow_runtime.rs:199`'s `// 4*19999 + (0+1+2+3)`, which was genuinely
    uncarried and which no gate flagged.

    Quote-aware and raw-string-aware, because the naive predicate misfires on
    both: a `"http://..."` inside a plain string, and any `//` inside a fixture
    body.
    """
    raw = _raw_string_spans(text)
    out = []
    for n, line in enumerate(text.split('\n'), 1):
        if n in raw:
            continue
        i, instr = 0, False
        while i < len(line):
            c = line[i]
            if instr:
                if c == '\\':
                    i += 2
                    continue
                if c == '"':
                    instr = False
                i += 1
                continue
            if c == '"':
                instr = True
                i += 1
                continue
            if c == '/' and i + 1 < len(line) and line[i + 1] == '/':
                if line[:i].strip():
                    body = line[i:].lstrip('/').strip()
                    if body and 'kali-tree-shake' not in body:
                        out.append((n, body))
                break
            i += 1
    return out


def extract_comment_paragraphs(text):
    lines = text.split('\n')
    paragraphs = []
    cur = []
    cur_start = None
    for i, line in enumerate(lines):
        m = re.match(r'^\s*//[!/]?\s?(.*)$', line)
        if m:
            if 'kali-tree-shake' in line:
                continue
            if cur_start is None:
                cur_start = i + 1
            cur.append(m.group(1))
        else:
            if cur:
                paragraphs.append((cur_start, cur))
                cur = []
                cur_start = None
    if cur:
        paragraphs.append((cur_start, cur))
    return paragraphs


def is_divider(p):
    return len(p) == 1 and re.match(r'^[-=]{3,}$', p[0].strip())


def normalize(s):
    return re.sub(r'\s+', ' ', s).strip()


def check(rs_path, toml_paths):
    """Returns (total_lines_checked, missing, n_cases), where `missing` is a
    list of (source_position, line_text, case_name) for every (paragraph
    line, case) pair where that case's OWN rationale does not contain the
    line's text. A case with no `rationale` at all is reported missing for
    every line (there is nothing to check membership against).

    `toml_paths` IS A LIST, and the whole list is one denominator. A U1/U2
    split puts one source's cases in two or more case files -- forced by
    matrix scope or by `[source]` being file-wide, never chosen -- and this
    gate used to take exactly one TOML, so such a pair could not be checked
    as a pair at all. Checking one half alone is not merely partial, it is
    WRONG in a specific direction: every comment belonging to the other
    half's helpers is reported "MISSING from ALL N cases", a hard red that
    no correct file can clear. U6's unit of attribution is the migrated
    SET -- a comment belongs in the rationale of exactly the cases its
    producing helper reaches, wherever those cases were forced to live -- so
    the set is what the denominator has to be. Case names are qualified with
    their file's stem once more than one file is in scope, because two case
    files may legitimately use the same case name.

    U10: a `#[path = "..."] mod ...;` carrier's prose can live in the carrier,
    in any submodule, or in both, and reading only the carrier would report a
    green (or, worse, a `--allow-empty` VACUOUS green) for a target whose entire
    comment budget sits one hop away. Every reachable submodule is scanned too.
    `source_position` is therefore `(basename, line)`, not a bare integer --
    a line number alone is ambiguous once more than one file is in scope, which
    is the same ambiguity that makes a `:N` citation into a carrier meaningless
    when the construct is in a submodule."""
    files = [Path(rs_path)] + submodule_paths(rs_path)
    if isinstance(toml_paths, (str, Path)):        # one-TOML callers still work
        toml_paths = [toml_paths]
    qualify = len(toml_paths) > 1

    case_blobs = []
    cases = []
    for toml_path in toml_paths:
        stem = Path(toml_path).stem
        doc = tomllib.load(open(toml_path, 'rb'))
        for case in doc.get('case', []) or []:
            cases.append(case)
            name = case.get('name', '<unnamed>')
            r = case.get('rationale') or ''
            case_blobs.append((f"{stem}::{name}" if qualify else name, normalize(r)))

    missing = []
    total = 0
    for path in files:
        src = path.read_text(encoding='utf-8')
        paragraphs = list(extract_comment_paragraphs(src))
        paragraphs += [(n, [body]) for n, body in extract_trailing_comments(src)]
        for start, para in paragraphs:
            if is_divider(para):
                continue
            for j, line in enumerate(para):
                line = line.strip()
                if not line:
                    continue
                total += 1
                norm = normalize(line)
                if not norm:
                    continue
                for case_name, blob in case_blobs:
                    if norm not in blob:
                        # (file, line, text) -- the FILE and LINE stay separate
                        # values so the report can sort NUMERICALLY. Batch 6B
                        # first shipped a pre-joined `"name:12"` string, which
                        # sorted lexicographically (1, 10, 100, ..., 2, 20) for
                        # every file in the corpus, `mod`-free ones included.
                        # Same content, same exit code, different reading order:
                        # a behaviour change where the commit claimed none.
                        missing.append(((path.name, start + j), line, case_name))
    return total, missing, len(cases)


def main() -> int:
    argv = sys.argv[1:]
    allow_empty = False
    if '--allow-empty' in argv:
        allow_empty = True
        argv = [a for a in argv if a != '--allow-empty']
    if len(argv) < 2:
        print("usage: comment_coverage.py [--allow-empty] SOURCE.rs "
              "TARGET.toml [TARGET.toml ...]", file=sys.stderr)
        return 64  # EX_USAGE -- distinct from 1 (missing) and 2 (vacuous)
    rs, tomls = argv[0], argv[1:]
    total, missing, n_cases = check(rs, tomls)
    # Group by (line, text) so a comment paragraph missing from every case
    # isn't printed N times; report the case COUNT it's missing from instead.
    by_line = {}
    for where, line, case_name in missing:
        by_line.setdefault((where, line), []).append(case_name)
    print(f"{total} non-divider comment lines checked against {n_cases} cases, "
          f"{len(by_line)} line(s) with at least one case missing them")
    for ((name, ln), line), case_names in sorted(by_line.items()):
        where = f"{name}:{ln}"
        if len(case_names) == n_cases:
            print(f"  MISSING {where} from ALL {n_cases} cases: {line!r}")
        else:
            print(f"  MISSING {where} from {len(case_names)}/{n_cases} cases "
                  f"(e.g. {case_names[0]!r}): {line!r}")
    # FIXED (Task 18 pilot review round 2, blocker C): this checker used to
    # report failures without ever calling sys.exit, so it always exited 0
    # -- a caller wiring it into a batch loop (as batches 2-8 will) would
    # read a red run as a pass. A checker that only reports is not a gate.
    if by_line:
        return 1
    # FIXED (Task 18 controller ruling 5, added by batch 3): a run that
    # checked ZERO non-divider comment lines used to exit 0, indistinguishable
    # from a run that checked 40 lines and found them all covered. That is a
    # vacuous green, and batches 3-8 are about to run this across ~133 files,
    # most of which will legitimately have no Rust comments at all -- exactly
    # the population in which "0 checked, exit 0" would silently become the
    # normal, unexamined result. So an empty check is now a distinct nonzero
    # exit (2), and a caller that has established by reading the source that
    # the file genuinely carries no prose must say so explicitly with
    # `--allow-empty`. Green then means "prose was checked and covered", and
    # nothing else can produce it by accident.
    if total == 0:
        if allow_empty:
            print("VACUOUS: 0 non-divider comment lines in the source; "
                  "emptiness explicitly acknowledged via --allow-empty")
            return 0
        print("VACUOUS: 0 non-divider comment lines checked -- this is not "
              "coverage. Confirm by reading the source that it truly carries "
              "no Rust comments, then re-run with --allow-empty.",
              file=sys.stderr)
        return 2
    return 0


if __name__ == '__main__':
    sys.exit(main())
