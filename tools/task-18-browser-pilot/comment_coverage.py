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


def check(rs_path, toml_path):
    """Returns (total_lines_checked, missing), where `missing` is a list of
    (source_position, line_text, case_name) for every (paragraph line, case)
    pair where that case's OWN rationale does not contain the line's text.
    A case with no `rationale` at all is reported missing for every line
    (there is nothing to check membership against).

    U10: a `#[path = "..."] mod ...;` carrier's prose can live in the carrier,
    in any submodule, or in both, and reading only the carrier would report a
    green (or, worse, a `--allow-empty` VACUOUS green) for a target whose entire
    comment budget sits one hop away. Every reachable submodule is scanned too.
    `source_position` is therefore `"<basename>:<line>"`, not a bare integer --
    a line number alone is ambiguous once more than one file is in scope, which
    is the same ambiguity that makes a `:N` citation into a carrier meaningless
    when the construct is in a submodule."""
    files = [Path(rs_path)] + submodule_paths(rs_path)
    doc = tomllib.load(open(toml_path, 'rb'))
    cases = doc.get('case', []) or []

    case_blobs = []
    for case in cases:
        r = case.get('rationale') or ''
        case_blobs.append((case.get('name', '<unnamed>'), normalize(r)))

    missing = []
    total = 0
    for path in files:
        src = path.read_text(encoding='utf-8')
        for start, para in extract_comment_paragraphs(src):
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
                        missing.append((f"{path.name}:{start + j}", line, case_name))
    return total, missing, len(cases)


def main() -> int:
    argv = sys.argv[1:]
    allow_empty = False
    if '--allow-empty' in argv:
        allow_empty = True
        argv = [a for a in argv if a != '--allow-empty']
    if len(argv) != 2:
        print("usage: comment_coverage.py [--allow-empty] SOURCE.rs TARGET.toml",
              file=sys.stderr)
        return 64  # EX_USAGE -- distinct from 1 (missing) and 2 (vacuous)
    rs, toml = argv[0], argv[1]
    total, missing, n_cases = check(rs, toml)
    # Group by (line, text) so a comment paragraph missing from every case
    # isn't printed N times; report the case COUNT it's missing from instead.
    by_line = {}
    for ln, line, case_name in missing:
        by_line.setdefault((ln, line), []).append(case_name)
    print(f"{total} non-divider comment lines checked against {n_cases} cases, "
          f"{len(by_line)} line(s) with at least one case missing them")
    for (ln, line), case_names in sorted(by_line.items()):
        if len(case_names) == n_cases:
            print(f"  MISSING {ln} from ALL {n_cases} cases: {line!r}")
        else:
            print(f"  MISSING {ln} from {len(case_names)}/{n_cases} cases "
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
