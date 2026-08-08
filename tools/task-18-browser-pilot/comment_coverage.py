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
import re
import sys
import tomllib


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
    (source_line_no, line_text, case_name) for every (paragraph line, case)
    pair where that case's OWN rationale does not contain the line's text.
    A case with no `rationale` at all is reported missing for every line
    (there is nothing to check membership against)."""
    src = open(rs_path, encoding='utf-8').read()
    paragraphs = extract_comment_paragraphs(src)
    doc = tomllib.load(open(toml_path, 'rb'))
    cases = doc.get('case', []) or []

    case_blobs = []
    for case in cases:
        r = case.get('rationale') or ''
        case_blobs.append((case.get('name', '<unnamed>'), normalize(r)))

    missing = []
    total = 0
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
                    missing.append((start + j, line, case_name))
    return total, missing, len(cases)


def main() -> int:
    rs, toml = sys.argv[1], sys.argv[2]
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
            print(f"  MISSING line {ln} from ALL {n_cases} cases: {line!r}")
        else:
            print(f"  MISSING line {ln} from {len(case_names)}/{n_cases} cases "
                  f"(e.g. {case_names[0]!r}): {line!r}")
    # FIXED (Task 18 pilot review round 2, blocker C): this checker used to
    # report failures without ever calling sys.exit, so it always exited 0
    # -- a caller wiring it into a batch loop (as batches 2-8 will) would
    # read a red run as a pass. A checker that only reports is not a gate.
    return 1 if by_line else 0


if __name__ == '__main__':
    sys.exit(main())
