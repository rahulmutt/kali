import re, sys, tomllib

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
    src = open(rs_path, encoding='utf-8').read()
    paragraphs = extract_comment_paragraphs(src)
    doc = tomllib.load(open(toml_path, 'rb'))
    blob = ''
    # include file-level '#' header comments too, and every rationale
    header_lines = []
    for line in open(toml_path, encoding='utf-8'):
        if line.startswith('#'):
            header_lines.append(line[1:].strip())
        elif line.strip() == '':
            continue
        else:
            break
    blob += normalize(' '.join(header_lines)) + ' \x00 '
    for case in doc.get('case', []):
        r = case.get('rationale')
        if r:
            blob += normalize(r) + ' \x00 '
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
            if norm and norm not in blob:
                missing.append((start + j, line))
    return total, missing

if __name__ == '__main__':
    rs, toml = sys.argv[1], sys.argv[2]
    total, missing = check(rs, toml)
    print(f"{total} non-divider comment lines checked, {len(missing)} missing")
    for ln, line in missing:
        print(f"  MISSING line {ln}: {line!r}")
