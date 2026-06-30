# kali_api_web src test-monolith modularization — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split kali_api_web's four multi-concern co-located `src/*_tests.rs` unit-test monoliths into thin facades + per-concern `#[path] mod` submodules, with zero behavior change and byte-identical `#[test]` bodies.

**Architecture:** Pure verbatim code-motion driven by a deterministic `#[test]`-fn mover (`.superpowers/sdd/move_fns.py`) with a string/comment/raw-string-aware brace lexer. Each facade drains to 0 module-level fns (retaining only its `use` lines), submodules sit under `src/<stem>_tests/<mod>.rs` headed by `use super::*;`, and product siblings keep their existing `#[cfg(test)] #[path] mod` decls untouched. A companion `verify.py` proves byte-identity per file.

**Tech Stack:** Rust (cargo, workspace crate `kali_api_web`), Python 3 (mover + verifier, git-ignored scratch under `.superpowers/sdd/`).

## Global Constraints

- **Verbatim only:** `#[test]` fn bodies move byte-for-byte; never rewrite, reformat, or re-path anything inside a moved block.
- **No `cargo fmt`** (repo `fmt --all --check` is already red on baseline; verbatim moves are not regressions).
- **No production-source edits, no `pub`/visibility widening, no public-API change, no `include_*!` path rewrites.** Consumers must compile unedited.
- **Mover invariants:** keep `FN_RE`, `IDENT_CHARS`, `find_close_line` byte-identical to the source embedded in Task 0; only the CLI args drive behavior.
- **Baseline (captured 2026-06-30):** `cargo build -p kali_api_web --tests` → **0 warnings**; `cargo test -p kali_api_web --lib` → **57 passed; 0 failed**. Per-file `--list`: threads **11**, worker **9**, events **6**, crypto **5**.
- **Integration:** branch `refactor/kali_api_web-modularization` off local `main`; ff-merge to local `main` only — **never push to origin**.
- All commands run from repo root unless stated; mover/verifier run from `crates/kali_api_web`.

---

### Task 0: Tooling + baseline + snapshots

**Files:**
- Create: `.superpowers/sdd/move_fns.py`
- Create: `.superpowers/sdd/verify.py`
- Create: `/tmp/claude-1000/-workspace/kali_api_web_split_scratch/orig/{threads,worker,events,crypto}_tests.rs` (pre-move snapshots)

**Interfaces:**
- Produces: `move_fns.py` CLI — `python3 move_fns.py <root_rs_relpath> "<groups-spec>" ["<pins>"]` (run from `crates/kali_api_web`); `verify.py` CLI — `python3 verify.py <orig_rs> "<submodule_glob>"`.

- [ ] **Step 1: Confirm baseline is green**

Run: `cargo build -p kali_api_web --tests 2>&1 | grep -c '^warning'` → expect `0`.
Run: `cargo test -p kali_api_web --lib 2>&1 | grep 'test result:'` → expect `57 passed; 0 failed`.

- [ ] **Step 2: Create `.superpowers/sdd/move_fns.py`** with exactly this content:

```python
#!/usr/bin/env python3
"""Verbatim #[test]-fn mover for the crate test-monolith modularization series.

Usage (run from the crate dir, e.g. crates/kali_api_web):
    python3 move_fns.py <root_rs_relpath> "<groups-spec>" ["<pin1,pin2,...>"]

groups-spec: "name=p1,p2;name2=p3;misc=*"
    - each group has one or more comma-separated leading-prefix matchers
    - a #[test] fn joins the FIRST group whose prefix-tuple its name starts with
    - "*" = catch-all (must be the last group); empty groups are auto-skipped
3rd arg (optional): comma-separated #[test] fn NAMES to PIN in the facade (not moved).

Splits the file's module-level #[test] fns into src/<stem>/<mod>.rs submodules
(each `use super::*;` + verbatim fn blocks), and rewrites the facade to drop the
moved fns and append `#[path] mod` decls. Non-#[test] module-level items stay put.
"""
import os
import re
import sys

IDENT_CHARS = set(
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_"
)
FN_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z0-9_]+)")


def find_close_line(lines, open_idx):
    """Return the index of the line holding the brace that closes the fn block
    opened on line `open_idx`. String/char/comment-aware, raw-string-aware so a
    `}` at column 0 inside an r#"..."# template is not miscounted."""
    depth = 0
    started = False
    i = open_idx
    while i < len(lines):
        line = lines[i]
        j = 0
        n = len(line)
        while j < n:
            c = line[j]
            two = line[j:j + 2]
            if two == "//":
                break  # rest of line is a comment
            if two == "/*":
                # block comment: scan to */ possibly across lines
                k = line.find("*/", j + 2)
                while k == -1:
                    i += 1
                    if i >= len(lines):
                        return len(lines) - 1
                    line = lines[i]
                    n = len(line)
                    k = line.find("*/", 0)
                j = k + 2
                continue
            if c == '"':
                # plain or raw string. detect leading r#* via lookback.
                hashes = 0
                p = j - 1
                while p >= 0 and line[p] == "#":
                    hashes += 1
                    p -= 1
                is_raw = p >= 0 and line[p] == "r"
                if is_raw:
                    closer = '"' + ("#" * hashes)
                    k = line.find(closer, j + 1)
                    while k == -1:
                        i += 1
                        if i >= len(lines):
                            return len(lines) - 1
                        line = lines[i]
                        n = len(line)
                        k = line.find(closer, 0)
                    j = k + len(closer)
                    continue
                else:
                    j += 1
                    while j < n:
                        if line[j] == "\\":
                            j += 2
                            continue
                        if line[j] == '"':
                            j += 1
                            break
                        j += 1
                    continue
            if c == "'":
                # char literal vs lifetime: lifetime is 'ident (no closing ')
                if j + 2 < n and line[j + 2] == "'":
                    j += 3
                    continue
                if j + 3 < n and line[j + 1] == "\\" and line[j + 3] == "'":
                    j += 4
                    continue
                j += 1
                continue
            if c == "{":
                depth += 1
                started = True
            elif c == "}":
                depth -= 1
                if started and depth == 0:
                    return i
            j += 1
        i += 1
    raise RuntimeError("unterminated fn starting at line %d" % open_idx)


def parse_groups(spec):
    groups = []  # list of (name, prefixes_tuple_or_STAR)
    for part in spec.split(";"):
        part = part.strip()
        if not part:
            continue
        name, _, prefs = part.partition("=")
        name = name.strip()
        prefs = prefs.strip()
        if prefs == "*":
            groups.append((name, "*"))
        else:
            groups.append((name, tuple(p for p in prefs.split(",") if p)))
    return groups


def group_for(fn_name, groups):
    for name, prefs in groups:
        if prefs == "*":
            return name
        if fn_name.startswith(prefs):
            return name
    return None


def main():
    root = sys.argv[1]
    groups = parse_groups(sys.argv[2])
    pins = set()
    if len(sys.argv) > 3 and sys.argv[3].strip():
        pins = {p.strip() for p in sys.argv[3].split(",") if p.strip()}

    with open(root, "r") as fh:
        text = fh.read()
    lines = text.split("\n")
    trailing_nl = text.endswith("\n")
    if trailing_nl:
        lines = lines[:-1]

    # Locate every module-level #[test] block.
    blocks = []  # (start_idx, end_idx, fn_name)
    i = 0
    while i < len(lines):
        if lines[i].strip() == "#[test]":
            k = i + 1
            while k < len(lines) and not FN_RE.match(lines[k]):
                k += 1
            if k >= len(lines):
                raise RuntimeError("#[test] at %d has no fn" % i)
            m = FN_RE.match(lines[k])
            fn_name = m.group(1)
            end = find_close_line(lines, k)
            blocks.append((i, end, fn_name))
            i = end + 1
        else:
            i += 1

    moved_idx = set()
    by_group = {}  # group_name -> list of block-text
    for (s, e, name) in blocks:
        if name in pins:
            continue
        g = group_for(name, groups)
        if g is None:
            raise RuntimeError("fn %s matched no group" % name)
        block_text = "\n".join(lines[s:e + 1])
        by_group.setdefault(g, []).append(block_text)
        for x in range(s, e + 1):
            moved_idx.add(x)

    # Facade = original lines minus moved blocks.
    facade = []
    i = 0
    while i < len(lines):
        if i in moved_idx:
            i += 1
            continue
        facade.append(lines[i])
        i += 1
    while facade and facade[-1].strip() == "":
        facade.pop()

    stem = os.path.splitext(os.path.basename(root))[0]
    subdir = os.path.join(os.path.dirname(root), stem)
    os.makedirs(subdir, exist_ok=True)

    decls = []
    for (gname, _) in groups:
        texts = by_group.get(gname)
        if not texts:
            continue  # empty group auto-skipped
        body = "use super::*;\n\n" + "\n\n".join(texts) + "\n"
        with open(os.path.join(subdir, gname + ".rs"), "w") as fh:
            fh.write(body)
        decls.append('#[path = "%s/%s.rs"]\nmod %s;' % (stem, gname, gname))

    facade_text = "\n".join(facade)
    if facade_text and not facade_text.endswith("\n"):
        facade_text += "\n"
    if decls:
        facade_text += "\n" + "\n\n".join(decls) + "\n"
    with open(root, "w") as fh:
        fh.write(facade_text)

    print("moved %d fns into %d submodules under %s/" % (
        sum(len(v) for v in by_group.values()),
        len([g for g in by_group if by_group[g]]),
        subdir,
    ))
    for (gname, _) in groups:
        if by_group.get(gname):
            print("  %s: %d" % (gname, len(by_group[gname])))
    if pins:
        print("  pinned in facade: %s" % ", ".join(sorted(pins)))


if __name__ == "__main__":
    main()
```

- [ ] **Step 3: Create `.superpowers/sdd/verify.py`** with exactly this content:

```python
#!/usr/bin/env python3
"""Byte-identity verifier for the test-monolith mover.

Usage:
    python3 verify.py <orig_rs> "<submodule_glob>" [<facade_glob_for_pins>]

Proves that the {name: body} map of #[test] fns extracted from <orig_rs> equals
the union of the maps extracted from the submodule files (and optional facade
files holding pinned tests). Exits non-zero on any name-set or body mismatch.
Reuses move_fns.py's lexer (FN_RE / IDENT_CHARS / find_close_line) verbatim.
"""
import glob
import sys

from move_fns import FN_RE, find_close_line  # noqa: F401


def extract(path):
    with open(path, "r") as fh:
        text = fh.read()
    lines = text.split("\n")
    if text.endswith("\n"):
        lines = lines[:-1]
    out = {}
    i = 0
    while i < len(lines):
        if lines[i].strip() == "#[test]":
            k = i + 1
            while k < len(lines) and not FN_RE.match(lines[k]):
                k += 1
            name = FN_RE.match(lines[k]).group(1)
            end = find_close_line(lines, k)
            out[name] = "\n".join(lines[i:end + 1])
            i = end + 1
        else:
            i += 1
    return out


def main():
    orig = extract(sys.argv[1])
    parts = {}
    for pat in sys.argv[2:]:
        for f in sorted(glob.glob(pat)):
            for name, body in extract(f).items():
                if name in parts:
                    print("DUPLICATE name across submodules: %s" % name)
                    sys.exit(1)
                parts[name] = body

    on, pn = set(orig), set(parts)
    if on != pn:
        print("NAME-SET MISMATCH")
        print("  only in orig:  %s" % sorted(on - pn))
        print("  only in parts: %s" % sorted(pn - on))
        sys.exit(1)
    bad = [n for n in on if orig[n] != parts[n]]
    if bad:
        print("BODY MISMATCH for: %s" % sorted(bad))
        sys.exit(1)
    print("PROOF OK: %d/%d #[test] bodies byte-identical" % (len(on), len(on)))


if __name__ == "__main__":
    main()
```

- [ ] **Step 4: Snapshot the four originals** (for verify.py)

Run:
```bash
mkdir -p /tmp/claude-1000/-workspace/kali_api_web_split_scratch/orig
cp crates/kali_api_web/src/{threads,worker,events,crypto}_tests.rs \
   /tmp/claude-1000/-workspace/kali_api_web_split_scratch/orig/
```

- [ ] **Step 5: Record per-file baseline `--list` name-sets**

Run (from repo root):
```bash
cargo test -p kali_api_web --lib -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort > /tmp/claude-1000/-workspace/kali_api_web_split_scratch/all_baseline.txt
wc -l /tmp/claude-1000/-workspace/kali_api_web_split_scratch/all_baseline.txt
```
Expected: `57` lines.

- [ ] **Step 6: Commit tooling note** (the `.superpowers/sdd/` dir is git-ignored scratch; nothing to commit here). Verify `git status --porcelain` shows no tracked changes. No commit.

---

### Task 1: Split `threads_tests.rs` (11 → topology/atomics/shared_array_buffer)

**Files:**
- Modify (drain to facade): `crates/kali_api_web/src/threads_tests.rs`
- Create: `crates/kali_api_web/src/threads_tests/{topology,atomics,shared_array_buffer}.rs`
- Untouched (verify no diff): `crates/kali_api_web/src/threads.rs`

**Interfaces:**
- Consumes: `move_fns.py`, `verify.py`, snapshot from Task 0.
- Produces: namespace `threads::threads_tests::{topology,atomics,shared_array_buffer}::<test>`; facade retains 3 `use` lines + 3 `#[path] mod` decls.

- [ ] **Step 1: Run the mover**

Run (from `crates/kali_api_web`):
```bash
python3 ../../.superpowers/sdd/move_fns.py src/threads_tests.rs \
  "topology=thread_runtime_topology_;atomics=atomics_;shared_array_buffer=shared_array_buffer_"
```
Expected: `topology: 7`, `atomics: 1`, `shared_array_buffer: 3`.

- [ ] **Step 2: Prove byte-identity**

Run (from `crates/kali_api_web`):
```bash
python3 ../../.superpowers/sdd/verify.py \
  /tmp/claude-1000/-workspace/kali_api_web_split_scratch/orig/threads_tests.rs \
  "src/threads_tests/*.rs"
```
Expected: `PROOF OK: 11/11 #[test] bodies byte-identical`.

- [ ] **Step 3: Confirm facade drained + sibling untouched**

Run (from repo root):
```bash
grep -c '#\[test\]' crates/kali_api_web/src/threads_tests.rs   # expect 0
grep -c '#\[path'   crates/kali_api_web/src/threads_tests.rs   # expect 3
git diff --quiet crates/kali_api_web/src/threads.rs && echo "sibling UNCHANGED"
```
Expected: `0`, `3`, `sibling UNCHANGED`. The facade must still begin with exactly:
```
use crate::*;
use kali_common::bytewise_shared_memory_is_lock_free;
use serde_json::Value;
```

- [ ] **Step 4: Build gate (0 warnings)**

Run: `cargo build -p kali_api_web --tests 2>&1 | grep -c '^warning'`
Expected: `0`.

- [ ] **Step 5: Test gate (count + name-set preserved)**

Run (from repo root):
```bash
cargo test -p kali_api_web --lib -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort \
  | diff - /tmp/claude-1000/-workspace/kali_api_web_split_scratch/all_baseline.txt
cargo test -p kali_api_web --lib 2>&1 | grep 'test result:'
```
Expected: empty diff; `57 passed; 0 failed`.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_api_web/src/threads_tests.rs crates/kali_api_web/src/threads_tests/
git commit -m "refactor(kali_api_web): split threads_tests.rs into per-concern test submodules [refactor]"
```

---

### Task 2: Split `worker_tests.rs` (9 → worker_stub/broadcast_channel)

**Files:**
- Modify (drain to facade): `crates/kali_api_web/src/worker_tests.rs`
- Create: `crates/kali_api_web/src/worker_tests/{worker_stub,broadcast_channel}.rs`
- Untouched (verify no diff): `crates/kali_api_web/src/worker.rs`

**Interfaces:**
- Produces: namespace `worker::worker_tests::{worker_stub,broadcast_channel}::<test>`; facade retains 2 `use` lines + 2 `#[path] mod` decls.

- [ ] **Step 1: Run the mover**

Run (from `crates/kali_api_web`):
```bash
python3 ../../.superpowers/sdd/move_fns.py src/worker_tests.rs \
  "worker_stub=worker_stub_;broadcast_channel=broadcast_channel_"
```
Expected: `worker_stub: 5`, `broadcast_channel: 4`.

- [ ] **Step 2: Prove byte-identity**

Run (from `crates/kali_api_web`):
```bash
python3 ../../.superpowers/sdd/verify.py \
  /tmp/claude-1000/-workspace/kali_api_web_split_scratch/orig/worker_tests.rs \
  "src/worker_tests/*.rs"
```
Expected: `PROOF OK: 9/9 #[test] bodies byte-identical`.

- [ ] **Step 3: Confirm facade drained + sibling untouched**

Run (from repo root):
```bash
grep -c '#\[test\]' crates/kali_api_web/src/worker_tests.rs   # expect 0
grep -c '#\[path'   crates/kali_api_web/src/worker_tests.rs   # expect 2
git diff --quiet crates/kali_api_web/src/worker.rs && echo "sibling UNCHANGED"
```
Expected: `0`, `2`, `sibling UNCHANGED`. Facade must begin with exactly:
```
use crate::*;
use serde_json::Value;
```

- [ ] **Step 4: Build gate (0 warnings)**

Run: `cargo build -p kali_api_web --tests 2>&1 | grep -c '^warning'`
Expected: `0`.

- [ ] **Step 5: Test gate (count + name-set preserved)**

Run (from repo root):
```bash
cargo test -p kali_api_web --lib -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort \
  | diff - /tmp/claude-1000/-workspace/kali_api_web_split_scratch/all_baseline.txt
cargo test -p kali_api_web --lib 2>&1 | grep 'test result:'
```
Expected: empty diff; `57 passed; 0 failed`.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_api_web/src/worker_tests.rs crates/kali_api_web/src/worker_tests/
git commit -m "refactor(kali_api_web): split worker_tests.rs into per-concern test submodules [refactor]"
```

---

### Task 3: Split `events_tests.rs` (6 → abort/event_target/custom_event)

**Files:**
- Modify (drain to facade): `crates/kali_api_web/src/events_tests.rs`
- Create: `crates/kali_api_web/src/events_tests/{abort,event_target,custom_event}.rs`
- Untouched (verify no diff): `crates/kali_api_web/src/events.rs`

**Interfaces:**
- Produces: namespace `events::events_tests::{abort,event_target,custom_event}::<test>`; facade retains 4 `use` lines + 3 `#[path] mod` decls.

- [ ] **Step 1: Run the mover**

Run (from `crates/kali_api_web`):
```bash
python3 ../../.superpowers/sdd/move_fns.py src/events_tests.rs \
  "abort=abort_;event_target=event_target_;custom_event=custom_event_"
```
Expected: `abort: 2`, `event_target: 3`, `custom_event: 1`.

- [ ] **Step 2: Prove byte-identity**

Run (from `crates/kali_api_web`):
```bash
python3 ../../.superpowers/sdd/verify.py \
  /tmp/claude-1000/-workspace/kali_api_web_split_scratch/orig/events_tests.rs \
  "src/events_tests/*.rs"
```
Expected: `PROOF OK: 6/6 #[test] bodies byte-identical`.

- [ ] **Step 3: Confirm facade drained + sibling untouched**

Run (from repo root):
```bash
grep -c '#\[test\]' crates/kali_api_web/src/events_tests.rs   # expect 0
grep -c '#\[path'   crates/kali_api_web/src/events_tests.rs   # expect 3
git diff --quiet crates/kali_api_web/src/events.rs && echo "sibling UNCHANGED"
```
Expected: `0`, `3`, `sibling UNCHANGED`. Facade must begin with exactly:
```
use crate::*;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
```

- [ ] **Step 4: Build gate (0 warnings)**

Run: `cargo build -p kali_api_web --tests 2>&1 | grep -c '^warning'`
Expected: `0`.

- [ ] **Step 5: Test gate (count + name-set preserved)**

Run (from repo root):
```bash
cargo test -p kali_api_web --lib -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort \
  | diff - /tmp/claude-1000/-workspace/kali_api_web_split_scratch/all_baseline.txt
cargo test -p kali_api_web --lib 2>&1 | grep 'test result:'
```
Expected: empty diff; `57 passed; 0 failed`.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_api_web/src/events_tests.rs crates/kali_api_web/src/events_tests/
git commit -m "refactor(kali_api_web): split events_tests.rs into per-concern test submodules [refactor]"
```

---

### Task 4: Split `crypto_tests.rs` (5 → randomness/subtle)

**Files:**
- Modify (drain to facade): `crates/kali_api_web/src/crypto_tests.rs`
- Create: `crates/kali_api_web/src/crypto_tests/{randomness,subtle}.rs`
- Untouched (verify no diff): `crates/kali_api_web/src/crypto.rs`

**Interfaces:**
- Produces: namespace `crypto::crypto_tests::{randomness,subtle}::<test>`; facade retains 1 `use` line + 2 `#[path] mod` decls. The `randomness` group uses two disjoint leading prefixes (`random_`, `crypto_facade_`); `crypto_facade_` does not match `subtle`'s `crypto_subtle_`, so the partition is clean and order-independent.

- [ ] **Step 1: Run the mover**

Run (from `crates/kali_api_web`):
```bash
python3 ../../.superpowers/sdd/move_fns.py src/crypto_tests.rs \
  "randomness=random_,crypto_facade_;subtle=crypto_subtle_"
```
Expected: `randomness: 3`, `subtle: 2`.

- [ ] **Step 2: Prove byte-identity**

Run (from `crates/kali_api_web`):
```bash
python3 ../../.superpowers/sdd/verify.py \
  /tmp/claude-1000/-workspace/kali_api_web_split_scratch/orig/crypto_tests.rs \
  "src/crypto_tests/*.rs"
```
Expected: `PROOF OK: 5/5 #[test] bodies byte-identical`.

- [ ] **Step 3: Confirm facade drained + sibling untouched**

Run (from repo root):
```bash
grep -c '#\[test\]' crates/kali_api_web/src/crypto_tests.rs   # expect 0
grep -c '#\[path'   crates/kali_api_web/src/crypto_tests.rs   # expect 2
git diff --quiet crates/kali_api_web/src/crypto.rs && echo "sibling UNCHANGED"
```
Expected: `0`, `2`, `sibling UNCHANGED`. Facade must begin with exactly:
```
use crate::*;
```

- [ ] **Step 4: Build gate (0 warnings)**

Run: `cargo build -p kali_api_web --tests 2>&1 | grep -c '^warning'`
Expected: `0`.

- [ ] **Step 5: Test gate (count + name-set preserved)**

Run (from repo root):
```bash
cargo test -p kali_api_web --lib -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort \
  | diff - /tmp/claude-1000/-workspace/kali_api_web_split_scratch/all_baseline.txt
cargo test -p kali_api_web --lib 2>&1 | grep 'test result:'
```
Expected: empty diff; `57 passed; 0 failed`.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_api_web/src/crypto_tests.rs crates/kali_api_web/src/crypto_tests/
git commit -m "refactor(kali_api_web): split crypto_tests.rs into per-concern test submodules [refactor]"
```

---

### Task 5: Whole-branch finalize + merge

**Files:** none (verification + integration only).

- [ ] **Step 1: Confirm changed-path inventory**

Run (from repo root):
```bash
git diff --name-status main...HEAD
```
Expected: exactly **4 modified facades** + **10 new submodules** (14 paths); no production `.rs`, no `Cargo.toml`, no out-of-scope `*_tests.rs`.

- [ ] **Step 2: Re-verify all four files byte-identical (aggregate)**

Run (from `crates/kali_api_web`):
```bash
for f in threads worker events crypto; do
  python3 ../../.superpowers/sdd/verify.py \
    /tmp/claude-1000/-workspace/kali_api_web_split_scratch/orig/${f}_tests.rs \
    "src/${f}_tests/*.rs"
done
```
Expected: four `PROOF OK` lines (11, 9, 6, 5).

- [ ] **Step 3: Final gates on the branch tip**

Run (from repo root):
```bash
cargo build -p kali_api_web --tests 2>&1 | grep -c '^warning'   # 0
cargo test -p kali_api_web --lib 2>&1 | grep 'test result:'     # 57 passed; 0 failed
```

- [ ] **Step 4: Opus whole-branch review**

Dispatch an opus reviewer over `git diff main...HEAD`: confirm line-conservation (every drained line reappears verbatim in a submodule; only added lines are `use super::*;` + blank + `#[path]`/`mod` scaffold), zero production/`pub`-widen/`include`/fmt changes, and the test name-multiset is conserved (57 base == 57 head, disjoint submodule namespaces, 0 collisions). Address any finding before merge.

- [ ] **Step 5: ff-merge to local main, re-verify, delete branch**

Run (from repo root):
```bash
git checkout main
git merge --ff-only refactor/kali_api_web-modularization
cargo build -p kali_api_web --tests 2>&1 | grep -c '^warning'   # 0
cargo test -p kali_api_web --lib 2>&1 | grep 'test result:'     # 57 passed; 0 failed
git branch -d refactor/kali_api_web-modularization
```
**Do NOT push to origin.** Then update the `crate-modularization-series` memory with the kali_api_web (34th) result.

---

## Self-Review

- **Spec coverage:** All 4 in-scope files (threads/worker/events/crypto) have a task (1–4); tooling + baseline (Task 0); finalize/merge/gates/review (Task 5). Out-of-scope files explicitly untouched (changed-path inventory in Task 5 Step 1). Gates, fmt policy, and no-origin-push constraint all carried into Global Constraints + tasks.
- **Placeholder scan:** No TBD/TODO; full tool source embedded; every command has expected output. The only delegated judgment is the opus review (Task 5 Step 4), which is a review gate, not a placeholder.
- **Type/name consistency:** groups-specs, prefixes, submodule names, namespaces, `#[path]`/`use`-line counts match the spec table and the proven scratch round-trip (threads 7/1/3, worker 5/4, events 2/3/1, crypto 3/2). `move_fns.py`/`verify.py` CLI signatures consistent across all tasks.
