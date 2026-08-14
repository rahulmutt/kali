r"""Independent per-#[test]-fn invocation inventory for Task 18 batch 3.

Written fresh for this batch (NOT a copy of the pilot's generators): reads a
`crates/kali_cli/tests/browser_*.rs` file, finds every `#[test] fn`, and prints
that fn's body verbatim with blank/`}` noise collapsed, so the batch's
invocation arithmetic (rule 7) can be re-derived by reading the real call sites
rather than by pattern-matching fn NAMES -- the near-miss the pilot report
records. Also prints, per file, the set of distinct helper names called from
#[test] bodies and a count.

Usage: inventory.py FILE.rs [FILE.rs ...]
"""
import re
import sys


def test_fns(text):
    lines = text.split('\n')
    out = []
    i = 0
    while i < len(lines):
        if lines[i].strip() == '#[test]':
            j = i + 1
            # fn signature may span lines; find the opening body brace line
            sig = []
            while j < len(lines):
                sig.append(lines[j])
                if lines[j].rstrip().endswith('{'):
                    break
                j += 1
            name = re.search(r'fn\s+([A-Za-z0-9_]+)', '\n'.join(sig))
            body = []
            depth = 1
            j += 1
            while j < len(lines) and depth > 0:
                depth += lines[j].count('{') - lines[j].count('}')
                if depth > 0:
                    body.append(lines[j])
                j += 1
            out.append((i + 1, name.group(1) if name else '?', body))
            i = j
            continue
        i += 1
    return out


def main():
    for path in sys.argv[1:]:
        text = open(path, encoding='utf-8').read()
        fns = test_fns(text)
        print(f'######## {path}: {len(fns)} #[test] fns')
        helpers = {}
        for line_no, name, body in fns:
            compact = ' '.join(l.strip() for l in body if l.strip())
            compact = re.sub(r'\s+', ' ', compact)
            print(f'  [{line_no}] {name}\n        {compact}')
            for h in re.findall(r'\b(assert_[a-z0-9_]+)\s*\(', compact):
                helpers[h] = helpers.get(h, 0) + 1
        print(f'  -- helper call sites (textual, per fn body): {helpers}')


if __name__ == '__main__':
    main()
